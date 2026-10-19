package prstest

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"sync"
	"time"
)

// Keep complete observation reports, including failed-prefix metadata. Core owns
// schema validation and the union; this adapter does not interpret coverage sets.
var suiteCoverage = struct {
	sync.Mutex
	Reports []json.RawMessage
}{Reports: []json.RawMessage{}}

func collectCoverage(data []byte) error {
	if !json.Valid(data) {
		return fmt.Errorf("invalid observation JSON")
	}
	suiteCoverage.Lock()
	defer suiteCoverage.Unlock()
	suiteCoverage.Reports = append(suiteCoverage.Reports, append(json.RawMessage(nil), data...))
	return nil
}

// WriteCoverage runs after m.Run, when all parallel tests and cleanups finished.
// Returning the tool error lets TestMain fail the test process on reporting errors.
func WriteCoverage(path string) error {
	suiteCoverage.Lock()
	defer suiteCoverage.Unlock()
	data, err := json.Marshal(suiteCoverage.Reports)
	if err != nil {
		return err
	}
	binary := os.Getenv("PRS_BIN")
	if binary == "" {
		binary = "prs"
	}
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()
	cmd := exec.CommandContext(ctx, binary, "coverage", "merge-observe", "--stdin", "--json-report", path)
	cmd.Stdin = bytes.NewReader(data)
	output, err := cmd.CombinedOutput()
	fmt.Print(string(output))
	if err != nil {
		return fmt.Errorf("P model coverage merge: %w", err)
	}
	fmt.Printf("P model coverage JSON: %s\n", path)
	return nil
}
