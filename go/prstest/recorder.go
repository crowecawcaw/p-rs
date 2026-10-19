// Package prstest checks events emitted by Go applications against P monitors
// using the separately installed prs executable. It integrates with testing.T,
// including subtests and fuzz callbacks, without process-global recording state.
package prstest

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"sync"
	"testing"
	"time"
)

// Config selects the monitors and controls one observation. Relative paths are
// resolved against WorkDir, or the working directory at construction time.
type Config struct {
	SpecFiles       []string
	Specs           []string
	Binary          string // Config, then PRS_BIN, then prs on PATH.
	WorkDir         string
	Timeout         time.Duration // Zero means 30 seconds; negative values are invalid.
	AllowUnobserved bool          // Opt out of the default --strict check.
	CheckHot        bool          // Check for monitors left in hot states at the end.
	ArtifactRoot    string        // Empty uses t.TempDir; otherwise retain a unique directory.
	Suite           *Suite        // Collect observed monitor coverage; leave nil in long fuzz campaigns.
}

// Result preserves the complete CLI report and diagnostics, including on failure.
// Report is nil when the executable could not produce a report.
type Result struct {
	Report       json.RawMessage
	Output       string
	ArtifactsDir string
}

type Recorder struct {
	mu       sync.Mutex
	verifyMu sync.Mutex
	cfg      Config
	dir      string
	trace    bytes.Buffer
	sticky   error
	sealed   bool
	done     bool
	result   Result
	err      error
}

func normalize(cfg Config) (Config, error) {
	if len(cfg.SpecFiles) == 0 || len(cfg.Specs) == 0 {
		return cfg, errors.New("SpecFiles and explicit Specs are required")
	}
	for _, spec := range cfg.SpecFiles {
		if spec == "" {
			return cfg, errors.New("empty spec file")
		}
	}
	seen := map[string]bool{}
	for _, name := range cfg.Specs {
		if name == "" || seen[name] {
			return cfg, fmt.Errorf("empty or duplicate spec name %q", name)
		}
		seen[name] = true
	}
	var err error
	cfg.WorkDir, err = filepath.Abs(cfg.WorkDir)
	if err != nil {
		return cfg, err
	}
	if cfg.Timeout < 0 {
		return cfg, errors.New("negative checker timeout")
	}
	if cfg.Timeout == 0 {
		cfg.Timeout = 30 * time.Second
	}
	cfg.SpecFiles = append([]string(nil), cfg.SpecFiles...)
	cfg.Specs = append([]string(nil), cfg.Specs...)
	for n, p := range cfg.SpecFiles {
		if !filepath.IsAbs(p) {
			cfg.SpecFiles[n] = filepath.Join(cfg.WorkDir, p)
		}
	}
	cfg.Binary = resolveBinary(cfg.Binary, cfg.WorkDir)
	if cfg.ArtifactRoot != "" && !filepath.IsAbs(cfg.ArtifactRoot) {
		cfg.ArtifactRoot = filepath.Join(cfg.WorkDir, cfg.ArtifactRoot)
	}
	return cfg, nil
}

func resolveBinary(binary, dir string) string {
	if binary == "" {
		binary = os.Getenv("PRS_BIN")
	}
	if binary == "" {
		binary = "prs"
	}
	if !filepath.IsAbs(binary) && strings.ContainsAny(binary, `/\`) {
		return filepath.Join(dir, binary)
	}
	return binary
}

// New constructs an explicit recorder without registering a verification cleanup.
// Configuration/resource errors fail the test immediately. Check or Verify after
// all application work (including workers) finishes.
func New(t testing.TB, cfg Config) *Recorder {
	t.Helper()
	cfg, err := normalize(cfg)
	if err != nil {
		t.Fatalf("prstest configuration: %v", err)
	}
	var dir string
	if cfg.ArtifactRoot == "" {
		dir = t.TempDir()
	} else {
		if err = os.MkdirAll(cfg.ArtifactRoot, 0o755); err != nil {
			t.Fatalf("prstest artifacts: %v", err)
		}
		name := strings.Map(func(r rune) rune {
			if r >= 'a' && r <= 'z' || r >= 'A' && r <= 'Z' || r >= '0' && r <= '9' || r == '-' {
				return r
			}
			return '_'
		}, t.Name())
		if len(name) > 80 {
			name = name[:80]
		}
		dir, err = os.MkdirTemp(cfg.ArtifactRoot, name+"-")
		if err != nil {
			t.Fatalf("prstest artifacts: %v", err)
		}
	}
	return &Recorder{cfg: cfg, dir: dir}
}

// Start registers a cleanup that verifies even if the test body fails. Register
// worker shutdown/join cleanups AFTER calling Start: Go runs them in reverse order.
// Each test/subtest/fuzz invocation must start its own recorder and application.
func Start(t testing.TB, cfg Config) *Recorder {
	t.Helper()
	r := New(t, cfg)
	t.Cleanup(func() { t.Helper(); r.Check(t) })
	return r
}

// Emit snapshots JSON-encodable payloads immediately. Unit events use nil;
// scalars, positional tuples (slices), and named tuples (maps/structs) are supported.
// Encoding errors are retained and fail verification rather than dropping events.
// The timestamp is actual Unix milliseconds sampled while holding the ordering
// lock. Backward clock changes are not hidden; use EmitAt for explicit event times.
func (r *Recorder) Emit(event, key string, payload any) {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.emit(time.Now().UnixMilli(), event, key, payload)
}

// EmitAt records the supplied event time as an ISO timestamp with nanosecond
// precision. Order is caller-controlled; events may be sorted by prs by timestamp.
func (r *Recorder) EmitAt(at time.Time, event, key string, payload any) {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.emit(at.UTC().Format(time.RFC3339Nano), event, key, payload)
}

func (r *Recorder) emit(timestamp any, event, key string, payload any) {
	if r.sealed {
		if r.sticky == nil {
			r.sticky = errors.New("event emitted after verification sealed the recorder; join workers before checking")
		}
		return
	}
	data, err := json.Marshal(map[string]any{"timestamp": timestamp, "event": event, "key": key, "payload": payload})
	if err != nil {
		if r.sticky == nil {
			r.sticky = fmt.Errorf("encode event %q: %w", event, err)
		}
		return
	}
	r.trace.Write(data)
	r.trace.WriteByte('\n')
}

// Verify seals the recorder and checks once. Repeated calls return independent
// copies of the cached report; later encoding/lifecycle errors still fail them.
// Concurrent Verify calls serialize. Do not emit concurrently with final checking.
func (r *Recorder) Verify(ctx context.Context) (Result, error) {
	r.verifyMu.Lock()
	defer r.verifyMu.Unlock()
	r.mu.Lock()
	r.sealed = true
	trace := append([]byte(nil), r.trace.Bytes()...)
	sticky := r.sticky
	r.mu.Unlock()
	wasDone := r.done
	if !r.done {
		r.done = true
		r.result = Result{ArtifactsDir: r.dir}
		r.err = r.verify(ctx, trace, sticky)
	}
	result := r.result
	result.Report = append(json.RawMessage(nil), result.Report...)
	if sticky != nil && wasDone {
		return result, errors.Join(r.err, fmt.Errorf("prstest: %w\nartifacts: %s", sticky, r.dir))
	}
	return result, r.err
}

func (r *Recorder) verify(ctx context.Context, trace []byte, sticky error) error {
	logPath := filepath.Join(r.dir, "events.jsonl")
	if err := os.WriteFile(logPath, trace, 0o600); err != nil {
		return fmt.Errorf("write events: %w", err)
	}
	if sticky != nil {
		return fmt.Errorf("prstest: %w\nartifacts: %s\nrecorded JSONL:\n%s", sticky, r.dir, trace)
	}
	args := []string{"observe", "-pf"}
	args = append(args, r.cfg.SpecFiles...)
	args = append(args, "--spec")
	args = append(args, r.cfg.Specs...)
	path := filepath.Join(r.dir, "observation.json")
	args = append(args, "-l", logPath, "--outputDir", filepath.Join(r.dir, "replay"), "--json-report", path)
	if !r.cfg.AllowUnobserved {
		args = append(args, "--strict")
	}
	if r.cfg.CheckHot {
		args = append(args, "--check-hot")
	}
	if r.cfg.Suite != nil {
		args = append(args, "--coverage")
	}
	ctx, cancel := context.WithTimeout(ctx, r.cfg.Timeout)
	defer cancel()
	cmd := exec.CommandContext(ctx, r.cfg.Binary, args...)
	cmd.Dir = r.cfg.WorkDir
	cmd.WaitDelay = time.Second
	output, commandErr := cmd.CombinedOutput()
	r.result.Output = string(output)
	data, readErr := os.ReadFile(path)
	r.result.Report = append(json.RawMessage(nil), data...)
	if ctx.Err() != nil {
		commandErr = errors.Join(commandErr, ctx.Err())
	}
	reportErr := validateReport(data, r.cfg.Specs, commandErr == nil, !r.cfg.AllowUnobserved)
	// Collect valid report JSON even when safety/strict validation failed. Core owns
	// coverage schema/fingerprint validation at merge time; never discard failed prefixes.
	if readErr == nil && reportErr == nil && r.cfg.Suite != nil {
		r.cfg.Suite.collect(data)
	}
	err := errors.Join(commandErr, readErr, reportErr)
	if err != nil {
		return fmt.Errorf("prstest observation: %w\ncommand: %s %q\n%s\nJSON report:\n%s\nartifacts: %s\nrecorded JSONL:\n%s", err, r.cfg.Binary, args, output, data, r.dir, trace)
	}
	return nil
}

// Check reports verification errors through the testing API.
func (r *Recorder) Check(t testing.TB) {
	t.Helper()
	if _, err := r.Verify(context.Background()); err != nil {
		t.Errorf("%v", err)
	}
}

func validateReport(data []byte, names []string, commandOK, strict bool) error {
	var report struct {
		Exit  *int `json:"exit_code"`
		Specs []struct {
			Name     string  `json:"spec"`
			Events   *uint64 `json:"events_read"`
			Keys     *uint64 `json:"partition_keys"`
			Verified *uint64 `json:"verified_keys"`
			Errors   []struct {
				Kind    string `json:"kind"`
				Message string `json:"message"`
			} `json:"errors"`
		} `json:"specs"`
	}
	if err := json.Unmarshal(data, &report); err != nil {
		return fmt.Errorf("decode observation report: %w", err)
	}
	if report.Exit == nil || len(report.Specs) != len(names) {
		return errors.New("missing report status or unexpected selected specs")
	}
	wanted := map[string]bool{}
	for _, name := range names {
		wanted[name] = true
	}
	hasErrors := false
	for _, s := range report.Specs {
		if !wanted[s.Name] {
			return fmt.Errorf("unexpected or duplicate report spec %q", s.Name)
		}
		delete(wanted, s.Name)
		if s.Events == nil || s.Keys == nil || s.Verified == nil || *s.Verified > *s.Keys || *s.Events < *s.Keys || s.Errors == nil {
			return fmt.Errorf("missing or invalid metrics/errors for %s", s.Name)
		}
		if *report.Exit == 0 {
			if (*s.Events == 0) != (*s.Keys == 0) {
				return fmt.Errorf("successful report has inconsistent event/partition emptiness for %s", s.Name)
			}
			if *s.Verified != *s.Keys {
				return fmt.Errorf("successful report left partitions unverified for %s", s.Name)
			}
			if strict && *s.Events == 0 {
				return fmt.Errorf("strict successful report checked no events for %s", s.Name)
			}
		}
		for _, e := range s.Errors {
			switch e.Kind {
			case "spec_violation", "out_of_order", "parser", "unknown", "validation":
			default:
				return fmt.Errorf("invalid report error kind %q", e.Kind)
			}
			if e.Message == "" {
				return errors.New("missing report error message")
			}
			hasErrors = true
		}
	}
	if *report.Exit < 0 || *report.Exit > 4 || (*report.Exit == 0) == hasErrors || (commandOK && *report.Exit != 0) || (!commandOK && *report.Exit == 0) {
		return fmt.Errorf("inconsistent process/report status %d", *report.Exit)
	}
	return nil
}
