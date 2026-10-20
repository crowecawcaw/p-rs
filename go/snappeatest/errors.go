package snappeatest

import (
	"bytes"
	"encoding/json"
	"fmt"
	"path/filepath"
	"regexp"
	"strings"
	"time"
)

// reportEvents is how many of the violating key's events a failure shows.
const reportEvents = 20

// Event is one recorded event, as shown in a SpecViolation or returned by an AddLog
// parser.
type Event struct {
	Event   string
	Key     string
	Payload any       // decoded JSON: map[string]any for a named tuple, nil for none
	Time    time.Time // when it happened; required from an AddLog parser
	Src     string    // file:line of the emit call, when known
	At      bool      // Time was given by EmitAt, not taken from the wall clock

	raw json.RawMessage
}

// SpecViolation is a P spec broken by the recorded events. Verify returns it (use
// errors.As); Check and Start's cleanup report it with t.Error.
type SpecViolation struct {
	Spec     string
	Key      string
	Message  string // the spec's assert message, or which hot state it ended in
	Location string // file:line of the P assert, when known
	Events   []Event
	Earlier  int  // events of Key before Events, not shown
	Liveness bool // the key ended in a hot state (CheckHot)
	Others   []*SpecViolation
	// ArtifactsDir holds the events, snappea output and report. It is kept after the test.
	ArtifactsDir string
}

// SetupError is a check that could not run or whose input is unusable. Kind is one of
// binary_missing, config, parser, validation, unknown_event, out_of_order, timeout,
// or another error kind the core reports.
type SetupError struct {
	Kind         string
	Message      string
	ArtifactsDir string
	Err          error // underlying cause, if any
}

func (e *SetupError) Error() string {
	s := fmt.Sprintf("snappeatest %s error: %s", e.Kind, e.Message)
	if e.ArtifactsDir != "" {
		s += "\n\n  snappea output: " + e.ArtifactsDir
	}
	return s
}

func (e *SetupError) Unwrap() error { return e.Err }

func (v *SpecViolation) Error() string {
	var b strings.Builder
	if v.Liveness {
		fmt.Fprintf(&b, "P spec %s was still waiting when the events ended: %s\n\n", v.Spec, v.Message)
	} else {
		fmt.Fprintf(&b, "P spec %s was broken: %s\n\n", v.Spec, v.Message)
	}
	if v.Key != "" {
		fmt.Fprintf(&b, "  Events (key %q):\n", v.Key)
	} else {
		b.WriteString("  Events:\n")
	}
	if v.Earlier > 0 {
		fmt.Fprintf(&b, "     ... %d earlier event(s)\n", v.Earlier)
	}
	wEvent, wPayload := 0, 0
	texts := make([]string, len(v.Events))
	for i, e := range v.Events {
		texts[i] = payloadText(e.raw)
		wEvent = max(wEvent, len(e.Event))
		wPayload = min(max(wPayload, len(texts[i])), 48)
	}
	for i, e := range v.Events {
		row := fmt.Sprintf("    %3d  %-*s  %-*s  %s", v.Earlier+i+1, wEvent, e.Event, wPayload, texts[i], e.Src)
		if e.At {
			row += "  (time set with EmitAt)"
		}
		if i == len(v.Events)-1 {
			if v.Liveness {
				row += "  <- events ended here"
			} else {
				row += "  <- broke the spec here"
			}
		} else {
			row = strings.TrimRight(row, " ")
		}
		b.WriteString(row + "\n")
	}
	if len(v.Events) == 0 {
		b.WriteString("    (none)\n")
	}
	b.WriteString("\n")
	if v.Location != "" {
		fmt.Fprintf(&b, "  Spec: %s\n", v.Location)
	}
	if v.ArtifactsDir != "" {
		fmt.Fprintf(&b, "  snappea output: %s\n", v.ArtifactsDir)
	}
	if len(v.Others) > 0 {
		fmt.Fprintf(&b, "\n  %d more violation(s):\n", len(v.Others))
		for _, o := range v.Others {
			key := ""
			if o.Key != "" {
				key = fmt.Sprintf(" (key %q)", o.Key)
			}
			fmt.Fprintf(&b, "    %s%s: %s\n", o.Spec, key, o.Message)
		}
	}
	return strings.TrimRight(b.String(), "\n")
}

// payloadText shows a named tuple as k=v in field order, other values as JSON.
func payloadText(raw json.RawMessage) string {
	raw = bytes.TrimSpace(raw)
	if len(raw) == 0 || string(raw) == "null" {
		return ""
	}
	if raw[0] != '{' {
		return short(string(raw))
	}
	dec := json.NewDecoder(bytes.NewReader(raw))
	if _, err := dec.Token(); err != nil {
		return short(string(raw))
	}
	var parts []string
	for dec.More() {
		k, err := dec.Token()
		var v json.RawMessage
		if err != nil || dec.Decode(&v) != nil {
			return short(string(raw))
		}
		text := string(v)
		var str string
		if json.Unmarshal(v, &str) == nil {
			text = str
		}
		parts = append(parts, fmt.Sprintf("%v=%s", k, short(text)))
	}
	return strings.Join(parts, " ")
}

func short(s string) string {
	if len(s) <= 40 {
		return s
	}
	return s[:37] + "..."
}

type reportError struct {
	Kind    string `json:"kind"`
	Key     string `json:"key"`
	Message string `json:"message"`
	Cause   string `json:"cause"`
	Events  []struct {
		Event   string `json:"event"`
		Key     string `json:"key"`
		LogLine string `json:"log_line"`
	} `json:"events"`
}

var (
	assertRe   = regexp.MustCompile(`(?s)^Assertion failure: (.+?:\d+):\d+ (.*)$`)
	livenessRe = regexp.MustCompile(`detected liveness bug in hot state '(.*)' at the end of program execution\.$`)
)

// classify turns a report into violations and the setup error that takes precedence
// over them, if any. Input errors (unknown events, unreadable lines, order) come first,
// since the events cannot be trusted; strict validation errors come only without
// violations.
func classify(data []byte, workDir, dir string) ([]*SpecViolation, *SetupError) {
	var report struct {
		Specs []struct {
			Spec   string        `json:"spec"`
			Errors []reportError `json:"errors"`
		} `json:"specs"`
	}
	if err := json.Unmarshal(data, &report); err != nil {
		return nil, &SetupError{Kind: "config", Message: "cannot read the snappea report: " + err.Error(), ArtifactsDir: dir}
	}
	byKind := map[string][]reportError{}
	var violations []*SpecViolation
	for _, s := range report.Specs {
		for _, e := range s.Errors {
			if e.Kind == "spec_violation" {
				violations = append(violations, violation(s.Spec, e, workDir, dir))
			} else {
				byKind[e.Kind] = append(byKind[e.Kind], e)
			}
		}
	}
	for _, kind := range []string{"unknown_event", "parser", "out_of_order", "unknown"} {
		if errs := byKind[kind]; len(errs) > 0 {
			return violations, &SetupError{Kind: kind, Message: inputMessage(kind, errs), ArtifactsDir: dir}
		}
	}
	if errs := byKind["validation"]; len(errs) > 0 && len(violations) == 0 {
		return nil, &SetupError{Kind: "validation", Message: inputMessage("validation", errs), ArtifactsDir: dir}
	}
	return violations, nil
}

func inputMessage(kind string, errs []reportError) string {
	var lines []string
	seen := map[string]bool{}
	if kind == "parser" {
		lines = append(lines, "some events do not match their declaration in the P spec files:")
	}
	for _, e := range errs {
		var line string
		if kind == "parser" {
			ev := eventOfLine(e.Message)
			where := ""
			if ev.Src != "" {
				where = " (" + ev.Src + ")"
			}
			cause := e.Cause
			if cause == "" {
				cause = "cannot be read"
			}
			if ev.Event == "" {
				line = fmt.Sprintf("  %s: %s", short(e.Message), cause)
			} else {
				line = fmt.Sprintf("  %s%s: %s\n    payload: %s", ev.Event, where, cause, short(string(ev.raw)))
			}
		} else {
			line = e.Message
		}
		if !seen[line] {
			seen[line] = true
			lines = append(lines, line)
		}
		if len(seen) == 10 {
			lines = append(lines, "  ...")
			break
		}
	}
	return strings.Join(lines, "\n")
}

func violation(spec string, e reportError, workDir, dir string) *SpecViolation {
	v := &SpecViolation{Spec: spec, Key: e.Key, Message: e.Message, ArtifactsDir: dir}
	if m := assertRe.FindStringSubmatch(e.Message); m != nil {
		v.Location, v.Message = relLocation(m[1], workDir), m[2]
	} else if m := livenessRe.FindStringSubmatch(e.Message); m != nil {
		v.Liveness = true
		v.Message = fmt.Sprintf("it ended in hot state '%s'", m[1])
	}
	for _, raw := range e.Events {
		ev := eventOfLine(raw.LogLine)
		if ev.Event == "" {
			ev.Event = raw.Event
		}
		if ev.Key == "" {
			ev.Key = raw.Key
		}
		v.Events = append(v.Events, ev)
	}
	if n := len(v.Events) - reportEvents; n > 0 {
		v.Earlier = n
		v.Events = v.Events[n:]
	}
	return v
}

func relLocation(loc, workDir string) string {
	i := strings.LastIndex(loc, ":")
	path, line := loc[:i], loc[i:]
	if filepath.IsAbs(path) {
		if rel, err := filepath.Rel(workDir, path); err == nil && !strings.HasPrefix(rel, "..") {
			path = rel
		}
	}
	return path + line
}

func eventOfLine(line string) Event {
	var d struct {
		Event     string          `json:"event"`
		Key       json.RawMessage `json:"key"`
		Timestamp json.RawMessage `json:"timestamp"`
		Payload   json.RawMessage `json:"payload"`
		Src       string          `json:"src"`
		At        bool            `json:"at"`
	}
	if json.Unmarshal([]byte(line), &d) != nil {
		return Event{}
	}
	ev := Event{Event: d.Event, Src: d.Src, At: d.At, raw: d.Payload}
	var key any
	if json.Unmarshal(d.Key, &key) == nil && key != nil {
		ev.Key = fmt.Sprint(key)
	}
	if len(d.Payload) > 0 {
		_ = json.Unmarshal(d.Payload, &ev.Payload)
	}
	var ts any
	if json.Unmarshal(d.Timestamp, &ts) == nil {
		switch t := ts.(type) {
		case string:
			ev.Time, _ = time.Parse(time.RFC3339Nano, t)
		case float64:
			ev.Time = time.UnixMilli(int64(t))
		}
	}
	return ev
}
