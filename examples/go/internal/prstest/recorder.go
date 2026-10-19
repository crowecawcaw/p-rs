// Package prstest is copyable test infrastructure, not a Go binding shipped by
// p-rs. It records JSONL and invokes the checkout's prs CLI once per test case.
package prstest

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"sync"
	"testing"
	"time"

	"example.com/inventory/inventory"
)

type Recorder struct {
	mu       sync.Mutex
	events   bytes.Buffer
	seq      int
	keys     map[string]bool
	coverage bool
}

func New() *Recorder { return &Recorder{keys: make(map[string]bool)} }

// Emit preserves emission order using deterministic logical timestamps. These
// specs check ordering, not elapsed time; there is no dependency on a wall clock.
func (r *Recorder) Emit(event, key string, payload map[string]any) {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.seq++
	r.keys[key] = true
	// The application emits only integer/string maps, so JSON encoding cannot fail.
	err := json.NewEncoder(&r.events).Encode(map[string]any{
		"event": event, "key": key, "payload": payload, "timestamp": r.seq,
	})
	if err != nil {
		panic(err)
	}
}

// Verify returns the CLI's full report, including the replay and the input trace
// on failure. Files and CLI output live under t.TempDir and are removed after
// cleanup; the Go test log retains the report and JSONL for CI diagnosis.
func (r *Recorder) Verify(t testing.TB) error {
	t.Helper()
	r.mu.Lock()
	trace := append([]byte(nil), r.events.Bytes()...)
	count, keys := r.seq, len(r.keys)
	r.mu.Unlock()
	if count == 0 {
		return fmt.Errorf("no events recorded: start recording before application setup")
	}
	dir := t.TempDir()
	logs := filepath.Join(dir, "events.jsonl")
	if err := os.WriteFile(logs, trace, 0o600); err != nil {
		return err
	}
	// Resolve against this helper's source, so every test package can call it.
	_, source, _, _ := runtime.Caller(0)
	spec := filepath.Join(filepath.Dir(source), "..", "..", "..", "python", "specs", "inventory.p")
	binary := os.Getenv("PRS_BIN")
	if binary == "" {
		binary = "prs"
	}
	reportPath := filepath.Join(dir, "result.json")
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()
	cmd := exec.CommandContext(ctx, binary, "observe", "-pf", spec,
		"--spec", "InventoryCorrect", "-l", logs, "--outputDir", filepath.Join(dir, "report"), "--json-report", reportPath)
	cmd.Args = append(cmd.Args, "--strict")
	if r.coverage {
		cmd.Args = append(cmd.Args, "--coverage")
	}
	output, commandErr := cmd.CombinedOutput()
	data, readErr := os.ReadFile(reportPath)
	if readErr != nil {
		return fmt.Errorf("read checker report (command error: %v): %w\n%s\nrecorded JSONL:\n%s", commandErr, readErr, output, trace)
	}
	var report struct {
		ExitCode int `json:"exit_code"`
		Specs    []struct {
			Spec     string            `json:"spec"`
			Events   int               `json:"events_read"`
			Keys     int               `json:"partition_keys"`
			Verified int               `json:"verified_keys"`
			Errors   []json.RawMessage `json:"errors"`
		} `json:"specs"`
	}
	if err := json.Unmarshal(data, &report); err != nil {
		return fmt.Errorf("decode checker report (command error: %v): %w\n%s", commandErr, err, output)
	}
	if len(report.Specs) != 1 {
		return fmt.Errorf("unexpected checker report: %s", data)
	}
	if r.coverage {
		if mergeErr := collectCoverage(data); mergeErr != nil {
			return mergeErr
		}
	}
	if commandErr != nil {
		return fmt.Errorf("InventoryCorrect: %w\n%s\nJSON report:\n%s\nrecorded JSONL:\n%s", commandErr, output, data, trace)
	}
	if report.ExitCode != 0 {
		return fmt.Errorf("unexpected checker report: %s", data)
	}
	s := report.Specs[0]
	if s.Spec != "InventoryCorrect" || s.Events != count || s.Keys != keys || s.Verified != keys || len(s.Errors) != 0 {
		return fmt.Errorf("checker did not verify all %d events and %d keys: %s", count, keys, data)
	}
	return nil
}

func (r *Recorder) Check(t testing.TB) {
	t.Helper()
	if err := r.Verify(t); err != nil {
		t.Fatal(err)
	}
}

// Checked starts observation before application setup. Its cleanup runs even if
// the test calls Fatal. Registered cleanups run last-in-first-out: register/join
// any application worker cleanup AFTER this call, so workers finish before check.
func Checked(t testing.TB) *inventory.Inventory {
	t.Helper()
	return checked(t, false)
}

// CheckedWithCoverage additionally retains each observation for WriteCoverage.
// Use this only in suites whose TestMain publishes a final report. Generated
// tests should use Checked so long fuzz campaigns retain no suite reports.
func CheckedWithCoverage(t testing.TB) *inventory.Inventory {
	t.Helper()
	return checked(t, true)
}

func checked(t testing.TB, coverage bool) *inventory.Inventory {
	t.Helper()
	r := New()
	r.coverage = coverage
	t.Cleanup(func() { r.Check(t) })
	return inventory.New(r.Emit)
}
