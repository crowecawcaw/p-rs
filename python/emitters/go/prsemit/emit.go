// Package prsemit lets a Go service emit events for P specs checked by the p-rs Python
// package (see docs/python.md). It is meant to be copied into a project: one file, no
// dependencies.
//
// When a test that is recording starts the service, PRS_EVENTS_DIR names a folder; each
// process appends JSON lines to a file of its own there. Without it, Emit does nothing.
package prsemit

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"sync"
	"time"
)

var (
	mu   sync.Mutex
	seq  int64
	last int64
)

// Emit records that event happened on stream key. payload is a struct or map whose JSON
// field names match the P event's payload type, or nil.
func Emit(event, key string, payload any) {
	emit(event, key, time.Time{}, payload)
}

// EmitAt is Emit with the time the event happened, when that differs from now.
func EmitAt(event, key string, at time.Time, payload any) {
	emit(event, key, at, payload)
}

func emit(event, key string, at time.Time, payload any) {
	dir := os.Getenv("PRS_EVENTS_DIR")
	if dir == "" {
		return
	}
	src := ""
	if _, file, line, ok := runtime.Caller(2); ok {
		src = fmt.Sprintf("%s:%d", filepath.Base(file), line)
	}
	mu.Lock()
	defer mu.Unlock()
	ns, overridden := at.UnixNano(), !at.IsZero()
	if !overridden {
		// never earlier than the previous event, so a clock step back keeps emit order
		ns = max(time.Now().UnixNano(), last)
		last = ns
	}
	seq++
	rec := map[string]any{
		"key":       key,
		"timestamp": time.Unix(0, ns).UTC().Format("2006-01-02T15:04:05.000000000Z"),
		"event":     event,
		"payload":   payload,
		"src":       src,
		"seq":       seq,
		"pid":       os.Getpid(),
	}
	if overridden {
		rec["at"] = true
	}
	line, err := json.Marshal(rec)
	if err != nil {
		return
	}
	name := filepath.Join(dir, fmt.Sprintf("events-go-%d.jsonl", os.Getpid()))
	f, err := os.OpenFile(name, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0o644)
	if err != nil {
		return
	}
	defer f.Close()
	f.Write(append(line, '\n'))
}
