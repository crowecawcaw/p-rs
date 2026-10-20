package snappeatest

import (
	"bufio"
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"
)

// TestConformance replays tests/conformance through Recorder.EmitAt and EventsDir;
// see tests/conformance/README.md.
func TestConformance(t *testing.T) {
	binary := snappeaBinary(t)
	root := filepath.Join("..", "..", "tests", "conformance")
	entries, err := os.ReadDir(root)
	if err != nil || len(entries) == 0 {
		t.Fatalf("no conformance cases in %s: %v", root, err)
	}
	for _, entry := range entries {
		if entry.IsDir() {
			dir := filepath.Join(root, entry.Name())
			t.Run(entry.Name(), func(t *testing.T) { replayCase(t, binary, dir) })
		}
	}
}

type counts struct {
	Covered *int `json:"covered"`
	Total   *int `json:"total"`
}

type conformanceCase struct {
	Description string   `json:"description"`
	Requires    []string `json:"requires"`
	Options     struct {
		Specs         []string `json:"specs"`
		Exclude       []string `json:"exclude"`
		Strict        *bool    `json:"strict"`
		CheckHot      bool     `json:"check_hot"`
		UnknownEvents string   `json:"unknown_events"`
	} `json:"options"`
	Expect struct {
		Outcome  string `json:"outcome"`
		Spec     string `json:"spec"`
		Key      string `json:"key"`
		Message  string `json:"message"`
		Liveness bool   `json:"liveness"`
		Kind     string `json:"kind"`
		Coverage map[string]struct {
			States   counts `json:"states"`
			Handlers counts `json:"handlers"`
		} `json:"coverage"`
	} `json:"expect"`
}

func replayCase(t *testing.T, binary, dir string) {
	data, err := os.ReadFile(filepath.Join(dir, "expect.json"))
	if err != nil {
		t.Fatal(err)
	}
	var c conformanceCase
	dec := json.NewDecoder(bytes.NewReader(data))
	dec.DisallowUnknownFields()
	if err := dec.Decode(&c); err != nil {
		t.Fatalf("expect.json: %v", err)
	}
	for _, feature := range c.Requires {
		if feature != "events_dir" && feature != "unknown_events" {
			t.Skipf("snappeatest lacks %s", feature)
		}
	}
	cfg := Config{
		Binary:        binary,
		Artifacts:     t.TempDir(),
		SpecFiles:     []string{filepath.Join(dir, "spec.p")},
		Specs:         c.Options.Specs,
		Exclude:       c.Options.Exclude,
		Strict:        c.Options.Strict,
		CheckHot:      c.Options.CheckHot,
		UnknownEvents: c.Options.UnknownEvents,
	}
	if c.Expect.Coverage != nil {
		cfg.Coverage = filepath.Join(t.TempDir(), "monitor.json")
	}
	r := New(t, cfg)

	events, err := os.Open(filepath.Join(dir, "events.jsonl"))
	if err != nil {
		t.Fatal(err)
	}
	defer events.Close()
	sc := bufio.NewScanner(events)
	for sc.Scan() {
		var e struct {
			Event     string          `json:"event"`
			Key       string          `json:"key"`
			Timestamp json.RawMessage `json:"timestamp"`
			Payload   json.RawMessage `json:"payload"`
		}
		if err := json.Unmarshal(sc.Bytes(), &e); err != nil {
			t.Fatalf("events.jsonl: %v", err)
		}
		var at time.Time
		var iso string
		var millis int64
		if json.Unmarshal(e.Timestamp, &iso) == nil {
			if at, err = time.Parse(time.RFC3339Nano, iso); err != nil {
				t.Fatal(err)
			}
		} else if json.Unmarshal(e.Timestamp, &millis) == nil {
			at = time.UnixMilli(millis)
		} else {
			t.Fatalf("timestamp %s", e.Timestamp)
		}
		r.EmitAt(at, e.Event, e.Key, e.Payload)
	}
	children, _ := os.ReadDir(filepath.Join(dir, "children"))
	for _, child := range children {
		data, err := os.ReadFile(filepath.Join(dir, "children", child.Name()))
		if err != nil {
			t.Fatal(err)
		}
		if err = os.WriteFile(filepath.Join(r.EventsDir(), child.Name()), data, 0o600); err != nil {
			t.Fatal(err)
		}
	}

	result, err := r.Verify(context.Background())
	want := c.Expect
	var v *SpecViolation
	var se *SetupError
	switch {
	case want.Outcome == "pass" && err != nil:
		t.Fatalf("want pass, got %v", err)
	case want.Outcome == "violation" && !errors.As(err, &v):
		t.Fatalf("want a violation, got %v", err)
	case want.Outcome == "violation":
		if v.Spec != want.Spec || want.Key != "" && v.Key != want.Key || !strings.Contains(v.Message, want.Message) || v.Liveness != want.Liveness {
			t.Fatalf("want %s on %q containing %q (liveness %t), got\n%v", want.Spec, want.Key, want.Message, want.Liveness, v)
		}
	case want.Outcome == "setup_error" && !errors.As(err, &se):
		t.Fatalf("want a %s setup error, got %v", want.Kind, err)
	case want.Outcome == "setup_error":
		if se.Kind != want.Kind || !strings.Contains(se.Message, want.Message) {
			t.Fatalf("want %s containing %q, got %v", want.Kind, want.Message, se)
		}
	}

	if want.Coverage == nil {
		return
	}
	var report struct {
		Specs []struct {
			Spec     string `json:"spec"`
			Coverage struct {
				States   counts `json:"states"`
				Handlers counts `json:"handlers"`
			} `json:"coverage"`
		} `json:"specs"`
	}
	if err := json.Unmarshal(result.Report, &report); err != nil {
		t.Fatalf("report: %v", err)
	}
	for spec, w := range want.Coverage {
		found := false
		for _, s := range report.Specs {
			if s.Spec != spec {
				continue
			}
			found = true
			for _, pair := range [][2]counts{{w.States, s.Coverage.States}, {w.Handlers, s.Coverage.Handlers}} {
				if !sameCount(pair[0].Covered, pair[1].Covered) || !sameCount(pair[0].Total, pair[1].Total) {
					t.Fatalf("%s coverage: want %+v, got %s", spec, w, result.Report)
				}
			}
		}
		if !found {
			t.Fatalf("no coverage for %s", spec)
		}
	}
}

func sameCount(want, got *int) bool {
	return want == nil || got != nil && *want == *got
}
