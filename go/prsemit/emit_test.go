package prsemit

import (
	"bytes"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"sync"
	"testing"
	"time"
)

type syncBuffer struct {
	mu  sync.Mutex
	buf bytes.Buffer
}

func (b *syncBuffer) Write(p []byte) (int, error) {
	b.mu.Lock()
	defer b.mu.Unlock()
	return b.buf.Write(p)
}

func decode(t *testing.T, data []byte) []map[string]any {
	t.Helper()
	var out []map[string]any
	for _, line := range bytes.Split(bytes.TrimSpace(data), []byte{'\n'}) {
		var m map[string]any
		if err := json.Unmarshal(line, &m); err != nil {
			t.Fatalf("line %q: %v", line, err)
		}
		out = append(out, m)
	}
	return out
}

// lineOf runs f, written on the caller's line, and returns that file:line.
func lineOf(f func()) string {
	f()
	_, file, line, _ := runtime.Caller(1)
	return fmt.Sprintf("%s:%d", filepath.Base(file), line)
}

func TestOffByDefault(t *testing.T) {
	if current.Load() != nil {
		t.Skip("PRS_EVENTS_DIR is set")
	}
	before := Dropped()
	Emit("e", "k", make(chan int)) // would be dropped if encoded
	if Dropped() != before {
		t.Fatal("off emitter encoded an event")
	}
}

func TestWriterSink(t *testing.T) {
	var out syncBuffer
	SetSink(Writer(&out))
	src := lineOf(func() { Emit("eReserved", "coffee", map[string]any{"order": "o1", "units": 2}) })
	EmitAt(time.Unix(5, 7), "eShipped", "coffee", nil)
	if !Flush(5 * time.Second) {
		t.Fatal("Flush timed out")
	}
	if n := len(decode(t, out.buf.Bytes())); n != 2 {
		t.Fatalf("flushed %d lines", n)
	}
	if err := Close(); err != nil {
		t.Fatal(err)
	}
	Emit("after", "k", nil) // off again
	got := decode(t, out.buf.Bytes())
	if len(got) != 2 {
		t.Fatalf("got %d lines", len(got))
	}
	if got[0]["event"] != "eReserved" || got[0]["key"] != "coffee" || got[0]["src"] != src || got[0]["at"] != nil || got[0]["pid"] != float64(os.Getpid()) {
		t.Fatalf("line: %v", got[0])
	}
	if got[1]["timestamp"] != "1970-01-01T00:00:05.000000007Z" || got[1]["at"] != true || got[1]["seq"].(float64) <= got[0]["seq"].(float64) {
		t.Fatalf("line: %v", got[1])
	}
}

func TestFileSinkAppends(t *testing.T) {
	path := filepath.Join(t.TempDir(), "events.jsonl")
	os.WriteFile(path, []byte(`{"event":"old","key":"","timestamp":1}`+"\n"), 0o600)
	s, err := File(path)
	if err != nil {
		t.Fatal(err)
	}
	SetSink(s)
	Emit("new", "k", 1)
	if err := Close(); err != nil {
		t.Fatal(err)
	}
	data, _ := os.ReadFile(path)
	if got := decode(t, data); len(got) != 2 || got[1]["event"] != "new" {
		t.Fatalf("file: %s", data)
	}
}

type blockingWriter struct{ release chan struct{} }

func (w blockingWriter) Write(p []byte) (int, error) { <-w.release; return len(p), nil }

type panickingWriter struct{}

func (panickingWriter) Write([]byte) (int, error) { panic("broken writer") }

func TestNeverBlocksOrPanics(t *testing.T) {
	w := blockingWriter{make(chan struct{})}
	SetSink(Writer(w))
	before := Dropped()
	done := make(chan struct{})
	go func() {
		for n := 0; n < QueueSize+100; n++ {
			Emit("e", "k", n)
		}
		close(done)
	}()
	select {
	case <-done:
	case <-time.After(10 * time.Second):
		t.Fatal("Emit blocked on a stuck writer")
	}
	if Dropped()-before < 99 {
		t.Fatalf("dropped %d, want at least 99", Dropped()-before)
	}
	close(w.release)
	Close()

	SetSink(Writer(panickingWriter{}))
	before = Dropped()
	Emit("e", "k", nil)
	Emit("bad", "k", make(chan int))
	Close()
	if Dropped()-before != 2 {
		t.Fatalf("dropped %d, want 2", Dropped()-before)
	}
}

func TestCallbackSink(t *testing.T) {
	var mu sync.Mutex
	var lines []string
	SetSink(Callback(func(line string) {
		mu.Lock()
		defer mu.Unlock()
		lines = append(lines, line)
	}))
	Emit("eStocked", "coffee", map[string]any{"units": 3})
	SetSink(nil) // replaces the sink: writes what was queued
	Emit("off", "k", nil)
	mu.Lock()
	defer mu.Unlock()
	if len(lines) != 1 || strings.HasSuffix(lines[0], "\n") || !strings.Contains(lines[0], `"eStocked"`) {
		t.Fatalf("lines: %q", lines)
	}
}

func TestEventsDirSink(t *testing.T) {
	dir := t.TempDir()
	s := &dirSink{dir: dir}
	var wg sync.WaitGroup
	for n := 0; n < 8; n++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			for j := 0; j < 50; j++ {
				s.write([]byte(fmt.Sprintf(`{"n":%d,"pad":"%s"}`+"\n", j, strings.Repeat("x", 500))))
			}
		}()
	}
	wg.Wait()
	files, _ := os.ReadDir(dir)
	if len(files) != 1 || !strings.HasPrefix(files[0].Name(), "prsemit.test-") || !strings.HasSuffix(files[0].Name(), ".jsonl") {
		t.Fatalf("files: %v", files)
	}
	data, _ := os.ReadFile(filepath.Join(dir, files[0].Name()))
	if got := decode(t, data); len(got) != 400 {
		t.Fatalf("lines: %d", len(got))
	}
}

func BenchmarkEmitOff(b *testing.B) {
	payload := map[string]any{"units": 1}
	for i := 0; i < b.N; i++ {
		Emit("eReserved", "coffee", payload)
	}
}
