// Package prsemit records application events for P specs, from application code.
//
// Call Emit where a change commits. Where the events go depends on the process:
//
//   - Started by a test recording (PRS_EVENTS_DIR set): each line is appended to this
//     process's own file in that folder, which the test checks
//     (docs/observe.md#events-folder-format).
//   - Production, after SetOutput or OpenFile: JSON lines that `prs observe -l` reads,
//     written by a background goroutine.
//   - Otherwise: nothing. Emit costs one atomic load.
//
// Emit never panics or blocks the caller: a line that cannot be encoded or written, or
// that finds the production queue full, is dropped and counted by Dropped.
// Standard library only.
package prsemit

import (
	"crypto/rand"
	"encoding/hex"
	"errors"
	"io"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"sync"
	"sync/atomic"
	"time"

	"github.com/crowecawcaw/p-rs/go/internal/eventline"
)

// EventsDirEnv names the folder a test recording gives the processes it starts.
const EventsDirEnv = "PRS_EVENTS_DIR"

// QueueSize is how many lines the production sink buffers before dropping.
const QueueSize = 4096

type sink interface {
	write(line []byte) bool
}

var (
	current atomic.Pointer[sink]
	dropped atomic.Uint64
	setMu   sync.Mutex
	testDir bool // PRS_EVENTS_DIR was set: SetOutput and OpenFile do nothing
	async   *asyncSink
)

func init() {
	if dir := os.Getenv(EventsDirEnv); dir != "" {
		testDir = true
		var s sink = &dirSink{dir: dir}
		current.Store(&s)
	}
}

// Emit records that event happened on stream key, now. payload is a struct or map whose
// JSON field names match the P event's named-tuple fields, a single value, or nil.
func Emit(event, key string, payload any) {
	if current.Load() == nil {
		return
	}
	emit(time.Time{}, event, key, payload)
}

// EmitAt is Emit with the time the event happened (a logical clock or an earlier time).
func EmitAt(at time.Time, event, key string, payload any) {
	if current.Load() == nil {
		return
	}
	if at.IsZero() {
		at = time.Unix(0, 0)
	}
	emit(at, event, key, payload)
}

func emit(at time.Time, event, key string, payload any) {
	defer func() {
		if recover() != nil {
			dropped.Add(1)
		}
	}()
	s := current.Load()
	if s == nil {
		return
	}
	src := eventline.Caller(2)
	n, ns := eventline.Stamp(at)
	line, err := eventline.Encode(ns, !at.IsZero(), event, key, payload, src, n)
	if err != nil || !(*s).write(line) {
		dropped.Add(1)
	}
}

// Dropped returns how many events could not be encoded or written since the process
// started.
func Dropped() uint64 { return dropped.Load() }

// SetOutput sends events to w as JSON lines from a background goroutine, replacing any
// earlier output; nil turns emitting off. Each line is one Write call. When w falls
// behind by QueueSize lines, further events are dropped. Under a test recording
// (PRS_EVENTS_DIR set) it does nothing: events go to the test.
func SetOutput(w io.Writer) { setOutput(w, nil) }

func setOutput(w io.Writer, closer io.Closer) {
	setMu.Lock()
	defer setMu.Unlock()
	if testDir {
		return
	}
	old := async
	async = nil
	if w == nil {
		current.Store(nil)
	} else {
		async = newAsync(w, closer)
		var s sink = async
		current.Store(&s)
	}
	if old != nil {
		old.close()
	}
}

// OpenFile appends events to path, creating it, as SetOutput does for a writer. Close
// flushes and closes it. Under a test recording it does nothing.
func OpenFile(path string) error {
	setMu.Lock()
	if testDir {
		setMu.Unlock()
		return nil
	}
	setMu.Unlock()
	f, err := os.OpenFile(path, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0o644)
	if err != nil {
		return err
	}
	setOutput(f, f)
	return nil
}

// Close writes the queued events, stops the production output and closes a file opened
// by OpenFile. Call it at shutdown. Events emitted afterwards are discarded.
func Close() error {
	setMu.Lock()
	defer setMu.Unlock()
	if testDir {
		return nil
	}
	current.Store(nil)
	if async == nil {
		return nil
	}
	err := async.close()
	async = nil
	return err
}

// dirSink appends to this process's own file in a test's events folder, one write per
// line, so a crash loses at most the line being written.
type dirSink struct {
	mu   sync.Mutex
	dir  string
	file *os.File
	err  bool
}

func (d *dirSink) write(line []byte) bool {
	d.mu.Lock()
	defer d.mu.Unlock()
	if d.file == nil && !d.err {
		f, err := os.OpenFile(filepath.Join(d.dir, fileName()), os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0o644)
		if err != nil {
			d.err = true
			return false
		}
		d.file = f
	}
	if d.file == nil {
		return false
	}
	_, err := d.file.Write(line)
	return err == nil
}

// fileName is <program>-<pid>-<random>.jsonl.
func fileName() string {
	name := strings.TrimSuffix(filepath.Base(os.Args[0]), ".exe")
	name = strings.Map(func(r rune) rune {
		if r >= 'a' && r <= 'z' || r >= 'A' && r <= 'Z' || r >= '0' && r <= '9' || r == '-' || r == '_' || r == '.' {
			return r
		}
		return '_'
	}, name)
	if name == "" || name == "." {
		name = "go"
	}
	var b [4]byte
	_, _ = rand.Read(b[:])
	return name + "-" + strconv.Itoa(os.Getpid()) + "-" + hex.EncodeToString(b[:]) + ".jsonl"
}

type asyncSink struct {
	lines  chan []byte
	done   chan struct{}
	closer io.Closer
	once   sync.Once
	err    error
}

func newAsync(w io.Writer, closer io.Closer) *asyncSink {
	a := &asyncSink{lines: make(chan []byte, QueueSize), done: make(chan struct{}), closer: closer}
	go a.run(w)
	return a
}

func (a *asyncSink) run(w io.Writer) {
	defer close(a.done)
	for line := range a.lines {
		if !writeLine(w, line) {
			dropped.Add(1)
		}
	}
}

func writeLine(w io.Writer, line []byte) (ok bool) {
	defer func() {
		if recover() != nil {
			ok = false
		}
	}()
	n, err := w.Write(line)
	return err == nil && n == len(line)
}

func (a *asyncSink) write(line []byte) (ok bool) {
	defer func() {
		if recover() != nil { // closed by a concurrent SetOutput or Close
			ok = false
		}
	}()
	select {
	case a.lines <- line:
		return true
	default:
		return false
	}
}

func (a *asyncSink) close() error {
	a.once.Do(func() {
		close(a.lines)
		<-a.done
		if a.closer != nil {
			a.err = a.closer.Close()
		}
	})
	if a.err != nil {
		return errors.Join(errors.New("prsemit: close output"), a.err)
	}
	return nil
}
