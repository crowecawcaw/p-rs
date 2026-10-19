package fuzz_test

import (
	"fmt"
	"testing"

	"example.com/inventory/internal/fixture"
)

var stock = map[string]int{"coffee": 3, "tea": 2}

func FuzzInventory(f *testing.F) {
	// Byte pairs encode action/product/order, then quantity. Seeds cover a
	// release, a rejected oversell, duplicate orders, and independent products.
	f.Add([]byte{0, 2, 1, 0, 2, 2})
	f.Add([]byte{0, 2, 4, 2, 0, 1, 2, 2})
	f.Fuzz(func(t *testing.T, actions []byte) {
		// Fresh inventory and recorder per call, including minimized replays.
		app := fixture.Checked(t)
		for _, sku := range []string{"coffee", "tea"} {
			if err := app.Stock(sku, stock[sku]); err != nil {
				t.Fatal(err)
			}
		}
		if len(actions) > 64 { // at most 32 operations per trace
			actions = actions[:64]
		}
		for n := 0; n+1 < len(actions); n += 2 {
			code := actions[n]
			sku := []string{"coffee", "tea"}[(code/2)%2]
			order := fmt.Sprintf("order-%d", (code/4)%4)
			if code%2 == 0 {
				_ = app.Reserve(sku, order, int(actions[n+1]%5)) // rejections are valid input
			} else {
				_ = app.Release(sku, order)
			}
			if got := app.Available(sku); got < 0 || got > stock[sku] {
				t.Fatalf("%s available = %d, outside [0,%d]", sku, got, stock[sku])
			}
		}
	})
}
