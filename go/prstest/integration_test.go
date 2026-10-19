package prstest

import (
	"bytes"
	"context"
	"encoding/json"
	"io"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"
)

const testSpec = `event eUnit;
event eCount: int;
event ePair: (int, string);
event eNamed: (units: int);
spec Values observes eUnit, eCount, ePair, eNamed {
 start state Ready {
  on eUnit do { }
  on eCount do (n: int) { assert n >= 0, "negative count"; }
  on ePair do (p: (int, string)) { assert p.0 == 7; assert p.1 == "hello"; }
  on eNamed do (p: (units: int)) { assert p.units >= 0; }
 }
}
spec Units observes eUnit { start state Ready { on eUnit do { } } }`

func integrationConfig(t *testing.T) Config {
	t.Helper()
	binary := os.Getenv("PRS_BIN")
	if binary == "" {
		var err error
		binary, err = exec.LookPath("prs")
		if err != nil {
			t.Skip("integration: build prs and set PRS_BIN to its absolute path")
		}
	}
	spec := filepath.Join(t.TempDir(), "values.p")
	if err := os.WriteFile(spec, []byte(testSpec), 0o600); err != nil {
		t.Fatal(err)
	}
	return Config{Binary: binary, SpecFiles: []string{spec}, Specs: []string{"Values"}}
}
func TestIntegrationPayloadsAndSubsetSpecs(t *testing.T) {
	cfg := integrationConfig(t)
	cfg.Specs = append(cfg.Specs, "Units")
	r := New(t, cfg)
	r.Emit("eUnit", "a", nil)
	r.Emit("eCount", "a", 42)
	r.Emit("ePair", "a", []any{7, "hello"})
	r.Emit("eNamed", "a", map[string]any{"units": 3})
	result, err := r.Verify(context.Background())
	if err != nil {
		t.Fatal(err)
	}
	result.Report[0] = '!'
	cached, err := r.Verify(context.Background())
	if err != nil || !json.Valid(cached.Report) {
		t.Fatalf("corrupted cache: %v", err)
	}
	r.Emit("eUnit", "a", nil)
	if _, err = r.Verify(context.Background()); err == nil || !strings.Contains(err.Error(), "sealed") {
		t.Fatalf("ignored late emission: %v", err)
	}
}
func TestIntegrationStrictAndOptOut(t *testing.T) {
	cfg := integrationConfig(t)
	result, err := New(t, cfg).Verify(context.Background())
	if err == nil || !bytes.Contains(result.Report, []byte(`"kind": "validation"`)) {
		t.Fatalf("empty trace not rejected: %v", err)
	}
	subset := cfg
	subset.Specs = []string{"Values", "Units"}
	onlyCounts := New(t, subset)
	onlyCounts.Emit("eCount", "k", 1)
	result, err = onlyCounts.Verify(context.Background())
	if err == nil || !bytes.Contains(result.Report, []byte(`"kind": "validation"`)) {
		t.Fatalf("strict mode ignored an unexercised selected spec: %v", err)
	}
	cfg.AllowUnobserved = true
	if _, err = New(t, cfg).Verify(context.Background()); err != nil {
		t.Fatal(err)
	}
}
func TestIntegrationCleanupAndIsolation(t *testing.T) {
	cfg := integrationConfig(t)
	root := t.TempDir()
	cfg.ArtifactRoot = root
	for n := 0; n < 8; n++ {
		t.Run("parallel", func(t *testing.T) {
			t.Parallel()
			r := Start(t, cfg)
			t.Cleanup(func() { r.Emit("eCount", "same-key", 1) })
		})
	}
	t.Cleanup(func() {
		entries, err := os.ReadDir(root)
		if err != nil || len(entries) != 8 {
			t.Errorf("want 8 isolated artifact directories; got %d: %v", len(entries), err)
		}
		for _, entry := range entries {
			data, err := os.ReadFile(filepath.Join(root, entry.Name(), "observation.json"))
			if err != nil || !json.Valid(data) {
				t.Errorf("missing observation: %s %v", entry.Name(), err)
			}
		}
	})
}
func TestIntegrationFailedCoverageAndMerge(t *testing.T) {
	cfg := integrationConfig(t)
	suite, err := NewSuite(SuiteConfig{Binary: cfg.Binary})
	if err != nil {
		t.Fatal(err)
	}
	cfg.Suite = suite
	r := New(t, cfg)
	r.Emit("eCount", "key", -1)
	result, err := r.Verify(context.Background())
	if err == nil || !bytes.Contains(result.Report, []byte(`"kind": "spec_violation"`)) {
		t.Fatalf("want actual spec violation: %v", err)
	}
	if _, err = r.Verify(context.Background()); err == nil {
		t.Fatal("lost cached failure")
	}
	suite.mu.Lock()
	count := len(suite.reports)
	suite.mu.Unlock()
	if count != 1 {
		t.Fatalf("collected %d reports", count)
	}
	path := filepath.Join(t.TempDir(), "model.json")
	var human bytes.Buffer
	if err = suite.WriteCoverage(context.Background(), path, &human); err != nil {
		t.Fatal(err)
	}
	data, err := os.ReadFile(path)
	if err != nil {
		t.Fatal(err)
	}
	if !bytes.Contains(data, []byte("negative count")) || !strings.Contains(human.String(), "Observed monitor coverage") {
		t.Fatalf("missing failed-prefix or human report: %s\n%s", data, human.String())
	}
	if err = suite.WriteCoverage(context.Background(), t.TempDir(), io.Discard); err == nil {
		t.Fatal("hidden coverage write error")
	}
	if err = os.WriteFile(cfg.SpecFiles[0], []byte(strings.Replace(testSpec, "negative count", "changed assertion", 1)), 0o600); err != nil {
		t.Fatal(err)
	}
	another := New(t, cfg)
	another.Emit("eCount", "key", 1)
	if _, err = another.Verify(context.Background()); err != nil {
		t.Fatal(err)
	}
	if err = suite.WriteCoverage(context.Background(), path, io.Discard); err == nil {
		t.Fatal("core accepted incompatible source fingerprints")
	}
}

func TestIntegrationParserErrorAndHotCompletion(t *testing.T) {
	t.Run("payload-mismatch", func(t *testing.T) {
		cfg := integrationConfig(t)
		r := New(t, cfg)
		r.Emit("eCount", "k", "invalid")
		result, err := r.Verify(context.Background())
		if err == nil || !bytes.Contains(result.Report, []byte(`"kind": "parser"`)) {
			t.Fatalf("parser failure hidden: %v", err)
		}
	})
	t.Run("hot-state", func(t *testing.T) {
		cfg := integrationConfig(t)
		source := `event e; spec Liveness observes e { start hot state Pending { on e do { } } }`
		if err := os.WriteFile(cfg.SpecFiles[0], []byte(source), 0o600); err != nil {
			t.Fatal(err)
		}
		cfg.Specs = []string{"Liveness"}
		cfg.CheckHot = true
		r := New(t, cfg)
		r.Emit("e", "k", nil)
		result, err := r.Verify(context.Background())
		if err == nil || !bytes.Contains(result.Report, []byte(`"kind": "spec_violation"`)) {
			t.Fatalf("hot completion failure hidden: %v", err)
		}
	})
}
