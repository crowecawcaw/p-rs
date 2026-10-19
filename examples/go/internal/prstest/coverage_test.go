package prstest

import (
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

// This adapter test has its own package test binary and aggregate. It does not
// add deliberately invalid events to the checked example's suite coverage.
func TestFailingObservationContributesCoverage(t *testing.T) {
	recorder := New()
	recorder.coverage = true
	recorder.Emit("eReserved", "coffee", map[string]any{"order": "alice", "units": 1})
	err := recorder.Verify(t)
	if err == nil || !strings.Contains(err.Error(), "exit status 1") || !strings.Contains(err.Error(), `"exit_code": 2`) {
		t.Fatalf("want nonzero checker failure, got %v", err)
	}
	// Repeated observations of an identical handler must not inflate coverage.
	suiteCoverage.Lock()
	duplicate := append([]byte(nil), suiteCoverage.Reports[len(suiteCoverage.Reports)-1]...)
	suiteCoverage.Unlock()
	if err := collectCoverage(duplicate); err != nil {
		t.Fatal(err)
	}
	path := filepath.Join(t.TempDir(), "coverage.json")
	if err := WriteCoverage(path); err != nil {
		t.Fatal(err)
	}
	data, err := os.ReadFile(path)
	if err != nil {
		t.Fatal(err)
	}
	var report struct {
		Specs []struct {
			Coverage struct {
				Handlers struct {
					Covered int               `json:"covered"`
					Visited []json.RawMessage `json:"visited"`
				} `json:"handlers"`
			} `json:"coverage"`
		} `json:"specs"`
	}
	if err := json.Unmarshal(data, &report); err != nil {
		t.Fatal(err)
	}
	if len(report.Specs) != 1 {
		t.Fatalf("missing failing check coverage: %s", data)
	}
	handlers := report.Specs[0].Coverage.Handlers
	if handlers.Covered != 1 || len(handlers.Visited) != 1 ||
		!strings.Contains(string(handlers.Visited[0]), "Unstocked") ||
		!strings.Contains(string(handlers.Visited[0]), "eReserved") {
		t.Fatalf("missing attempted handler: %s", data)
	}
}

func TestCoverageWriteErrorIsNotHidden(t *testing.T) {
	if err := WriteCoverage(t.TempDir()); err == nil {
		t.Fatal("expected coverage report write error")
	}
}
