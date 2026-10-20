// Package snappeaemit records application events for P specs, from application code.
//
// Call Emit where a change commits. Where the events go depends on the process:
//
//   - Inside a test with a snappeatest recording: to that recording. With several
//     recordings active at once (parallel tests), use EmitCtx with a context from
//     Recorder.Context; an Emit they could each own fails them all.
//   - Started by a test recording (SNAPPEA_EVENTS_DIR set): each line is appended to this
//     process's own file in that folder, which the test checks
//     (docs/observe.md#events-folder-format).
//   - Production, after SetSink(File(path)), SetSink(Stdout()), SetSink(Writer(w)) or
//     SetSink(Callback(f)): JSON lines that `snappea observe` reads, written by a
//     background goroutine.
//   - Otherwise: nothing. Emit costs one atomic load.
//
// Emit never panics or blocks the caller: a line that cannot be encoded or written, or
// that finds the production queue full, is dropped and counted by Dropped.
// Standard library only.
package snappeaemit

import (
	"context"
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

	"github.com/crowecawcaw/snappea/go/internal/eventline"
	"github.com/crowecawcaw/snappea/go/internal/hook"
)

// EventsDirEnv names the folder a test recording gives the processes it starts.
const EventsDirEnv = "SNAPPEA_EVENTS_DIR"

// QueueSize is how many lines the production sink buffers before dropping.
const QueueSize = 4096

type sink interface {
	write(line []byte) bool
}

var (
	current atomic.Pointer[sink]
	dropped atomic.Uint64
	setMu   sync.Mutex
	testDir bool // SNAPPEA_EVENTS_DIR was set: SetSink does nothing
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
	if current.Load() == nil && !hook.Active() {
		return
	}
	emit(nil, time.Time{}, event, key, payload)
}

// EmitAt is Emit with the time the event happened (a logical clock or an earlier time).
func EmitAt(at time.Time, event, key string, payload any) {
	if current.Load() == nil && !hook.Active() {
		return
	}
	if at.IsZero() {
		at = time.Unix(0, 0)
	}
	emit(nil, at, event, key, payload)
}

// EmitCtx is Emit that sends the event to the test recording attached to ctx
// (Recorder.Context in snappeatest), so parallel tests each get their own events. Without
// one attached, it is Emit.
func EmitCtx(ctx context.Context, event, key string, payload any) {
	if current.Load() == nil && !hook.Active() {
		return
	}
	emit(ctx, time.Time{}, event, key, payload)
}

func emit(ctx context.Context, at time.Time, event, key string, payload any) {
	defer func() {
		if recover() != nil {
			dropped.Add(1)
		}
	}()
	src := eventline.Caller(2)
	if hook.Active() && hook.Dispatch(ctx, at, src, event, key, payload) {
		return
	}
	s := current.Load()
	if s == nil {
		return
	}
	n, ns := eventline.Stamp(at)
	line, err := eventline.Encode(ns, !at.IsZero(), event, key, payload, src, n)
	if err != nil || !(*s).write(line) {
		dropped.Add(1)
	}
}

// Dropped returns how many events could not be encoded or written since the process
// started.
func Dropped() uint64 { return dropped.Load() }

// Sink is a production destination for events: JSON lines that `snappea observe` reads.
// Make one with File, Stdout, Writer or Callback and install it with SetSink.
type Sink struct {
	w      io.Writer
	closer io.Closer
}

// File appends events to path, creating it and its folder.
func File(path string) (*Sink, error) {
	if dir := filepath.Dir(path); dir != "" {
		if err := os.MkdirAll(dir, 0o755); err != nil {
			return nil, err
		}
	}
	f, err := os.OpenFile(path, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0o644)
	if err != nil {
		return nil, err
	}
	return &Sink{w: f, closer: f}, nil
}

// Stdout writes events to standard output.
func Stdout() *Sink { return &Sink{w: os.Stdout} }

// Writer writes events to w, one Write call per line. Close does not close w.
func Writer(w io.Writer) *Sink { return &Sink{w: w} }

// Callback calls f with each JSON line (without the newline), from the sink's
// background goroutine. A panic in f drops that event.
func Callback(f func(line string)) *Sink {
	return &Sink{w: callbackWriter(f)}
}

type callbackWriter func(string)

func (f callbackWriter) Write(p []byte) (int, error) {
	f(strings.TrimSuffix(string(p), "\n"))
	return len(p), nil
}

// SetSink sends events to s outside tests, from a background goroutine, replacing and
// closing any earlier sink; nil turns production emitting off. When s falls behind by
// QueueSize lines, further events are dropped. In a process a test started
// (SNAPPEA_EVENTS_DIR set) it does nothing: events go to the test.
func SetSink(s *Sink) {
	setMu.Lock()
	defer setMu.Unlock()
	if testDir {
		if s != nil && s.closer != nil {
			_ = s.closer.Close()
		}
		return
	}
	old := async
	async = nil
	if s == nil {
		current.Store(nil)
	} else {
		async = newAsync(s.w, s.closer)
		var out sink = async
		current.Store(&out)
	}
	if old != nil {
		_ = old.close()
	}
}

// Flush waits up to timeout for queued events to be written. It reports whether they
// were.
func Flush(timeout time.Duration) bool {
	setMu.Lock()
	a := async
	setMu.Unlock()
	if a == nil {
		return true
	}
	deadline := time.Now().Add(timeout)
	for a.pending.Load() > 0 {
		if time.Now().After(deadline) {
			return false
		}
		time.Sleep(time.Millisecond)
	}
	return true
}

// Close writes the queued events, turns production emitting off and closes a file
// opened by File. Call it at shutdown. Events emitted afterwards are discarded.
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
	lines   chan []byte
	pending atomic.Int64
	done    chan struct{}
	closer  io.Closer
	once    sync.Once
	err     error
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
		a.pending.Add(-1)
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
		if recover() != nil { // closed by a concurrent SetSink or Close
			a.pending.Add(-1)
			ok = false
		}
	}()
	a.pending.Add(1)
	select {
	case a.lines <- line:
		return true
	default:
		a.pending.Add(-1)
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
		return errors.Join(errors.New("snappeaemit: close sink"), a.err)
	}
	return nil
}
