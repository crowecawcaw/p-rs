// Package hook connects prsemit to the test recordings prstest starts in the same
// process, without prsemit depending on prstest.
package hook

import (
	"context"
	"sync"
	"sync/atomic"
	"time"
)

// Recording receives events from prsemit.
type Recording interface {
	// Record adds an event; at is zero for the wall clock.
	Record(at time.Time, src, event, key string, payload any)
	// Ambiguous reports an event that n concurrently active recordings could each own.
	Ambiguous(n int, src, event string)
}

var (
	count  atomic.Int32
	mu     sync.Mutex
	active []Recording
)

// Register makes r receive prsemit events until Unregister.
func Register(r Recording) {
	mu.Lock()
	defer mu.Unlock()
	active = append(active, r)
	count.Store(int32(len(active)))
}

func Unregister(r Recording) {
	mu.Lock()
	defer mu.Unlock()
	for i, a := range active {
		if a == r {
			active = append(active[:i:i], active[i+1:]...)
			break
		}
	}
	count.Store(int32(len(active)))
}

// Active reports, with one atomic load, whether any recording is registered.
func Active() bool { return count.Load() > 0 }

// Dispatch sends an event to the recording attached to ctx, else to the only active
// recording. With several and none attached, each is told the event is ambiguous.
// It reports whether any recording was active.
func Dispatch(ctx context.Context, at time.Time, src, event, key string, payload any) bool {
	if ctx != nil {
		if r, ok := ctx.Value(ctxKey{}).(Recording); ok {
			r.Record(at, src, event, key, payload)
			return true
		}
	}
	mu.Lock()
	rs := append([]Recording(nil), active...)
	mu.Unlock()
	switch len(rs) {
	case 0:
		return false
	case 1:
		rs[0].Record(at, src, event, key, payload)
	default:
		for _, r := range rs {
			r.Ambiguous(len(rs), src, event)
		}
	}
	return true
}

type ctxKey struct{}

// WithRecording attaches r to ctx, for prsemit.EmitCtx.
func WithRecording(ctx context.Context, r Recording) context.Context {
	return context.WithValue(ctx, ctxKey{}, r)
}
