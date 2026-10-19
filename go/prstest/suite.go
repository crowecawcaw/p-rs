package prstest

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"os"
	"os/exec"
	"path/filepath"
	"sync"
	"time"
)

// SuiteConfig configures the core coverage merger independently of recorders.
// Use the same CLI version for observation and merging.
type SuiteConfig struct {
	Binary  string
	WorkDir string
	Timeout time.Duration // Zero means 30 seconds.
}

// Suite explicitly retains observation reports, including failing prefixes.
// There are no global collectors. Do not attach a suite to unbounded fuzz runs.
type Suite struct {
	mu      sync.Mutex
	cfg     SuiteConfig
	reports []json.RawMessage
}

func NewSuite(cfg SuiteConfig) (*Suite, error) {
	var err error
	cfg.WorkDir, err = filepath.Abs(cfg.WorkDir)
	if err != nil {
		return nil, err
	}
	if cfg.Timeout < 0 {
		return nil, errors.New("negative coverage timeout")
	}
	if cfg.Timeout == 0 {
		cfg.Timeout = 30 * time.Second
	}
	cfg.Binary = resolveBinary(cfg.Binary, cfg.WorkDir)
	return &Suite{cfg: cfg, reports: []json.RawMessage{}}, nil
}

func (s *Suite) collect(data []byte) {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.reports = append(s.reports, append(json.RawMessage(nil), data...))
}

// WriteCoverage snapshots collected reports and invokes prs coverage merge-observe.
// Call after m.Run, when tests and their cleanups finish. Core validates schemas,
// source fingerprints and coverage unions, and retains failure metadata. A merge
// success is not a successful test result: preserve the exit code from m.Run.
// jsonPath is relative to SuiteConfig.WorkDir; human receives the text report.
func (s *Suite) WriteCoverage(ctx context.Context, jsonPath string, human io.Writer) error {
	if jsonPath == "" {
		return errors.New("coverage JSON path is required")
	}
	if human == nil {
		return errors.New("coverage human output writer is required")
	}
	s.mu.Lock()
	data, err := json.Marshal(s.reports)
	s.mu.Unlock()
	if err != nil {
		return err
	}
	if !filepath.IsAbs(jsonPath) {
		jsonPath = filepath.Join(s.cfg.WorkDir, jsonPath)
	}
	// Merge beside the destination and publish only after all checks succeed.
	// A failed merger or malformed output must not destroy an earlier report.
	if err = os.MkdirAll(filepath.Dir(jsonPath), 0o755); err != nil {
		return err
	}
	pending, err := os.CreateTemp(filepath.Dir(jsonPath), ".prstest-coverage-*.json")
	if err != nil {
		return err
	}
	pendingPath := pending.Name()
	defer os.Remove(pendingPath)
	if err = pending.Close(); err != nil {
		return err
	}
	ctx, cancel := context.WithTimeout(ctx, s.cfg.Timeout)
	defer cancel()
	cmd := exec.CommandContext(ctx, s.cfg.Binary, "coverage", "merge-observe", "--stdin", "--json-report", pendingPath)
	cmd.Dir = s.cfg.WorkDir
	cmd.Stdin = bytes.NewReader(data)
	cmd.WaitDelay = time.Second
	output, commandErr := cmd.CombinedOutput()
	written, writeErr := human.Write(output)
	if writeErr == nil && written != len(output) {
		writeErr = io.ErrShortWrite
	}
	if ctx.Err() != nil {
		commandErr = errors.Join(commandErr, ctx.Err())
	}
	if err = errors.Join(commandErr, writeErr); err != nil {
		return fmt.Errorf("observed monitor coverage merge: %w\n%s", err, output)
	}
	report, err := os.ReadFile(pendingPath)
	if err != nil {
		return fmt.Errorf("read merged coverage: %w\n%s", err, output)
	}
	var envelope struct {
		Scope        string            `json:"scope"`
		Version      *int              `json:"schema_version"`
		Specs        []json.RawMessage `json:"specs"`
		Observations []json.RawMessage `json:"observations"`
	}
	if err = json.Unmarshal(report, &envelope); err != nil {
		return fmt.Errorf("invalid merged coverage JSON: %w\n%s", err, report)
	}
	if envelope.Scope != "observed_monitor_suite" || envelope.Version == nil || *envelope.Version != 1 || envelope.Specs == nil || envelope.Observations == nil {
		return fmt.Errorf("invalid merged coverage envelope: %s", report)
	}
	if err = os.Rename(pendingPath, jsonPath); err != nil {
		return fmt.Errorf("publish merged coverage: %w", err)
	}
	return nil
}
