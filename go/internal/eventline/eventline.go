// Package eventline encodes lines of the events folder format
// (docs/observe.md#events-folder-format), shared by snappeaemit and snappeatest so both
// number and time events from one process consistently.
package eventline

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
	pid  = os.Getpid()
)

// Stamp returns the next per-process sequence number and, unless at is set, a
// wall-clock time in nanoseconds that never decreases within the process.
func Stamp(at time.Time) (n int64, ns int64) {
	mu.Lock()
	defer mu.Unlock()
	seq++
	if !at.IsZero() {
		return seq, at.UnixNano()
	}
	last = max(time.Now().UnixNano(), last)
	return seq, last
}

// Timestamp formats ns as ISO-8601 UTC with nanoseconds.
func Timestamp(ns int64) string {
	return time.Unix(0, ns).UTC().Format("2006-01-02T15:04:05.000000000Z")
}

type line struct {
	Key       string `json:"key"`
	Timestamp string `json:"timestamp"`
	Event     string `json:"event"`
	Payload   any    `json:"payload"`
	Src       string `json:"src,omitempty"`
	Seq       int64  `json:"seq,omitempty"`
	Pid       int    `json:"pid,omitempty"`
	At        bool   `json:"at,omitempty"`
}

// Encode returns one JSON line, ending in '\n'.
func Encode(ns int64, at bool, event, key string, payload any, src string, n int64) ([]byte, error) {
	data, err := json.Marshal(line{key, Timestamp(ns), event, payload, src, n, pid, at})
	if err != nil {
		return nil, err
	}
	return append(data, '\n'), nil
}

// Caller returns "file:line" of the caller skip frames above Caller's caller.
func Caller(skip int) string {
	if _, file, n, ok := runtime.Caller(skip + 1); ok {
		return fmt.Sprintf("%s:%d", filepath.Base(file), n)
	}
	return ""
}
