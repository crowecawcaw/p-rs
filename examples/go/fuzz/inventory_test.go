package fuzz_test

import (
	"fmt"
	"testing"

	"example.com/inventory/internal/prstest"
)

func FuzzInventory(f *testing.F) {
	// stock=3; pairs encode action/product/order and quantity. Seeds cover a
	// release, a rejected oversell, duplicate orders, and independent products.
	f.Add([]byte{0, 2, 1, 0, 2, 2})
	f.Add([]byte{0, 2, 4, 2, 0, 1, 2, 2})
	f.Fuzz(func(t *testing.T, actions []byte) {
		// Each invocation, including a minimized replay, gets fresh app, recorder,
		// temp files and P monitor instances. No package globals or environment mutation.
		app := prstest.Checked(t)
		for _, sku := range []string{"coffee", "tea"} {
			if err := app.Stock(sku, 3); err != nil {
				t.Fatal(err)
			}
		}
		// Bound each trace to 32 operations; native fuzzing still mutates sequences.
		if len(actions) > 64 {
			actions = actions[:64]
		}
		for n := 0; n+1 < len(actions); n += 2 {
			code := actions[n]
			sku := []string{"coffee", "tea"}[(code/2)%2]
			order := fmt.Sprintf("order-%d", (code/4)%4)
			if code%2 == 0 {
				_ = app.Reserve(sku, order, int(actions[n+1]%5))
			} else {
				_ = app.Release(sku, order)
			}
			// Rejections are valid inputs; P checks successful emitted changes.
			// Ordinary assertions cover state changes absent from the event stream.
			if got := app.Available(sku); got < 0 || got > 3 {
				t.Fatalf("%s available = %d, outside [0,3]", sku, got)
			}
		}
	})
}
