package prstest

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"strings"
	"sync"
	"testing"
	"time"
)

// Exercise command boundaries without a shell or the real CLI's failure modes.
func init() {
	mode := os.Getenv("PRSTEST_CHILD")
	if mode == "" {
		return
	}
	if mode == "timeout" {
		time.Sleep(2 * time.Second)
		os.Exit(0)
	}
	for n, arg := range os.Args {
		if arg == "--json-report" && n+1 < len(os.Args) {
			data := []byte(`{"exit_code":0,"specs":[]}`)
			switch mode {
			case "malformed":
				data = []byte(`{broken`)
			case "empty-object":
				data = []byte(`{}`)
			case "null":
				data = []byte(`null`)
			case "array":
				data = []byte(`[]`)
			case "unverified-partition":
				data = []byte(`{"exit_code":0,"specs":[{"spec":"S","events_read":1,"partition_keys":1,"verified_keys":0,"errors":[]}]}`)
			case "empty-observation":
				data = []byte(`{"exit_code":0,"specs":[{"spec":"S","events_read":0,"partition_keys":0,"verified_keys":0,"errors":[]}]}`)
			case "events-without-keys":
				data = []byte(`{"exit_code":0,"specs":[{"spec":"S","events_read":1,"partition_keys":0,"verified_keys":0,"errors":[]}]}`)
			case "keys-without-events":
				data = []byte(`{"exit_code":0,"specs":[{"spec":"S","events_read":0,"partition_keys":1,"verified_keys":1,"errors":[]}]}`)
			case "bad-envelope":
				data = []byte(`{"scope":"observed_monitor_suite","schema_version":1,"specs":[],"observations":null}`)
			}
			if err := os.WriteFile(os.Args[n+1], data, 0o600); err != nil {
				fmt.Fprintln(os.Stderr, err)
				os.Exit(2)
			}
		}
	}
	fmt.Println("fixture diagnostics")
	os.Exit(0)
}
func fixtureConfig(t *testing.T) Config {
	t.Helper()
	binary, err := os.Executable()
	if err != nil {
		t.Fatal(err)
	}
	return Config{Binary: binary, SpecFiles: []string{"unused.p"}, Specs: []string{"S"}}
}
func TestEmissionSnapshotAndTime(t *testing.T) {
	r := New(t, fixtureConfig(t))
	payload := map[string]any{"value": 1}
	before := time.Now().UnixMilli()
	r.EmitAt(time.Unix(10, 123), "tuple", "key", []any{1, "hello"})
	r.Emit("named", "key", payload)
	payload["value"] = 99
	r.Emit("scalar", "key", 3)
	r.Emit("unit", "key", nil)
	after := time.Now().UnixMilli()
	lines := bytes.Split(bytes.TrimSpace(r.trace.Bytes()), []byte{'\n'})
	if len(lines) != 4 {
		t.Fatalf("got %d events", len(lines))
	}
	for n, line := range lines {
		var event struct {
			Timestamp json.RawMessage
			Payload   json.RawMessage
		}
		if err := json.Unmarshal(line, &event); err != nil {
			t.Fatal(err)
		}
		if n == 0 {
			if string(event.Timestamp) != `"1970-01-01T00:00:10.000000123Z"` || string(event.Payload) != `[1,"hello"]` {
				t.Fatalf("lost precise tuple: %s", line)
			}
		} else {
			var stamp int64
			if err := json.Unmarshal(event.Timestamp, &stamp); err != nil {
				t.Fatal(err)
			}
			if stamp < before || stamp > after {
				t.Fatalf("not Unix millis: %s", line)
			}
		}
	}
	if !bytes.Contains(lines[1], []byte(`"value":1`)) || !bytes.Contains(lines[2], []byte(`"payload":3`)) || !bytes.Contains(lines[3], []byte(`"payload":null`)) {
		t.Fatalf("payload not snapshotted: %s", r.trace.Bytes())
	}
}
func TestConcurrentEmission(t *testing.T) {
	r := New(t, fixtureConfig(t))
	var workers sync.WaitGroup
	for n := 0; n < 16; n++ {
		workers.Add(1)
		go func(n int) {
			defer workers.Done()
			for j := 0; j < 16; j++ {
				r.Emit("e", "k", []int{n, j})
			}
		}(n)
	}
	workers.Wait()
	lines := bytes.Split(bytes.TrimSpace(r.trace.Bytes()), []byte{'\n'})
	if len(lines) != 256 {
		t.Fatalf("lost emissions: %d", len(lines))
	}
	for _, line := range lines {
		if !json.Valid(line) {
			t.Fatalf("interleaved JSON: %s", line)
		}
	}
}
func TestEncodingErrorIsSticky(t *testing.T) {
	r := New(t, fixtureConfig(t))
	r.Emit("bad", "k", make(chan int))
	r.Emit("fine", "k", nil)
	for n := 0; n < 2; n++ {
		result, err := r.Verify(context.Background())
		if err == nil || !strings.Contains(err.Error(), "encode event") {
			t.Fatalf("lost encoding error: %v", err)
		}
		if len(result.Report) != 0 {
			t.Fatal("checker ran after encoding failure")
		}
	}
}
func TestMalformedAndMismatchedToolReports(t *testing.T) {
	for _, mode := range []string{"malformed", "missing-spec"} {
		t.Run(mode, func(t *testing.T) {
			t.Setenv("PRSTEST_CHILD", mode)
			r := New(t, fixtureConfig(t))
			r.Emit("e", "k", nil)
			result, err := r.Verify(context.Background())
			if err == nil || !strings.Contains(err.Error(), "report") || !strings.Contains(result.Output, "fixture diagnostics") {
				t.Fatalf("lost malformed tool diagnostics: %v %+v", err, result)
			}
		})
	}
}
func TestTimeoutAndMissingBinary(t *testing.T) {
	t.Run("timeout", func(t *testing.T) {
		t.Setenv("PRSTEST_CHILD", "timeout")
		cfg := fixtureConfig(t)
		cfg.Timeout = 25 * time.Millisecond
		r := New(t, cfg)
		r.Emit("e", "k", nil)
		_, err := r.Verify(context.Background())
		if !errors.Is(err, context.DeadlineExceeded) {
			t.Fatalf("lost timeout: %v", err)
		}
	})
	t.Run("missing", func(t *testing.T) {
		cfg := fixtureConfig(t)
		cfg.Binary = filepath.Join(t.TempDir(), "missing")
		_, err := New(t, cfg).Verify(context.Background())
		if err == nil || !strings.Contains(err.Error(), cfg.Binary) {
			t.Fatalf("missing executable did not fail: %v", err)
		}
	})
}
func TestConfiguration(t *testing.T) {
	for _, cfg := range []Config{{}, {SpecFiles: []string{"a"}}, {SpecFiles: []string{"a"}, Specs: []string{"S", "S"}}, {SpecFiles: []string{"a"}, Specs: []string{"S"}, Timeout: -1}} {
		if _, err := normalize(cfg); err == nil {
			t.Fatalf("accepted invalid config: %+v", cfg)
		}
	}
	original := Config{SpecFiles: []string{"spec.p"}, Specs: []string{"S"}, WorkDir: t.TempDir(), Binary: "./prs"}
	normalized, err := normalize(original)
	if err != nil {
		t.Fatal(err)
	}
	original.SpecFiles[0] = "mutated"
	original.Specs[0] = "mutated"
	if !filepath.IsAbs(normalized.Binary) || !filepath.IsAbs(normalized.SpecFiles[0]) || normalized.Specs[0] != "S" {
		t.Fatalf("incorrect snapshot: %+v", normalized)
	}
}
func TestReportValidation(t *testing.T) {
	valid := `{"exit_code":0,"specs":[{"spec":"S","events_read":1,"partition_keys":1,"verified_keys":1,"errors":[]}]}`
	if err := validateReport([]byte(valid), []string{"S"}, true, true); err != nil {
		t.Fatal(err)
	}
	for _, bad := range []string{strings.Replace(valid, `"S"`, `"other"`, 1), strings.Replace(valid, `"verified_keys":1`, `"verified_keys":2`, 1), strings.Replace(valid, `"errors":[]`, `"errors":null`, 1), strings.Replace(valid, `"exit_code":0`, `"exit_code":2`, 1), `{"specs":[]}`, `{"exit_code":0,"specs":[{"spec":"S"}]}`} {
		if err := validateReport([]byte(bad), []string{"S"}, true, true); err == nil {
			t.Fatalf("accepted invalid report: %s", bad)
		}
	}
}

type shortWriter struct{}

func (shortWriter) Write(p []byte) (int, error) { return 0, nil }

func TestSuiteToolAndOutputFailures(t *testing.T) {
	binary, err := os.Executable()
	if err != nil {
		t.Fatal(err)
	}
	t.Setenv("PRSTEST_CHILD", "malformed")
	suite, err := NewSuite(SuiteConfig{Binary: binary})
	if err != nil {
		t.Fatal(err)
	}
	path := filepath.Join(t.TempDir(), "report.json")
	if err = suite.WriteCoverage(context.Background(), path, &bytes.Buffer{}); err == nil || !strings.Contains(err.Error(), "invalid merged coverage JSON") {
		t.Fatalf("accepted malformed merge report: %v", err)
	}
	if err = suite.WriteCoverage(context.Background(), path, shortWriter{}); !errors.Is(err, io.ErrShortWrite) {
		t.Fatalf("lost human output error: %v", err)
	}
	t.Setenv("PRSTEST_CHILD", "timeout")
	suite, err = NewSuite(SuiteConfig{Binary: binary, Timeout: 25 * time.Millisecond})
	if err != nil {
		t.Fatal(err)
	}
	if err = suite.WriteCoverage(context.Background(), path, &bytes.Buffer{}); !errors.Is(err, context.DeadlineExceeded) {
		t.Fatalf("lost coverage timeout cause: %v", err)
	}
}

func TestInvalidMergePreservesPreviousDestination(t *testing.T) {
	binary, err := os.Executable()
	if err != nil {
		t.Fatal(err)
	}
	suite, err := NewSuite(SuiteConfig{Binary: binary})
	if err != nil {
		t.Fatal(err)
	}
	path := filepath.Join(t.TempDir(), "monitor.json")
	previous := []byte("previous successful coverage")
	for _, mode := range []string{"malformed", "empty-object", "null", "array", "bad-envelope"} {
		t.Run(mode, func(t *testing.T) {
			t.Setenv("PRSTEST_CHILD", mode)
			if err := os.WriteFile(path, previous, 0o600); err != nil {
				t.Fatal(err)
			}
			if err := suite.WriteCoverage(context.Background(), path, io.Discard); err == nil {
				t.Fatalf("accepted invalid suite report from %s", mode)
			}
			got, err := os.ReadFile(path)
			if err != nil || !bytes.Equal(got, previous) {
				t.Fatalf("destroyed previous destination: %s %v", got, err)
			}
			entries, err := os.ReadDir(filepath.Dir(path))
			if err != nil || len(entries) != 1 {
				t.Fatalf("leaked merge staging file: %v %v", entries, err)
			}
		})
	}
}

// Even a well-shaped, exit-zero report must honor the core metric invariants.
// Opting out of strict observation permits emptiness, never skipped partitions.
func TestToolSuccessMetricInvariants(t *testing.T) {
	for _, mode := range []string{"unverified-partition", "empty-observation", "keys-without-events", "events-without-keys"} {
		for _, allow := range []bool{false, true} {
			t.Run(fmt.Sprintf("%s/allow-unobserved=%t", mode, allow), func(t *testing.T) {
				t.Setenv("PRSTEST_CHILD", mode)
				cfg := fixtureConfig(t)
				cfg.AllowUnobserved = allow
				_, err := New(t, cfg).Verify(context.Background())
				wantSuccess := allow && mode == "empty-observation"
				if (err == nil) != wantSuccess {
					t.Fatalf("success=%t, want %t: %v", err == nil, wantSuccess, err)
				}
			})
		}
	}
}
