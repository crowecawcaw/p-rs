package explicit_test

import (
	"strings"
	"testing"

	"example.com/inventory/internal/prstest"
	"example.com/inventory/inventory"
)

func TestReservationAndCancellation(t *testing.T) {
	t.Parallel()
	r := prstest.New()
	app := inventory.New(r.Emit)
	if err := app.Stock("coffee", 3); err != nil {
		t.Fatal(err)
	}
	if err := app.Reserve("coffee", "order-1", 2); err != nil {
		t.Fatal(err)
	}
	if err := app.Release("coffee", "order-1"); err != nil {
		t.Fatal(err)
	}
	if got := app.Available("coffee"); got != 3 {
		t.Fatalf("available = %d, want 3", got)
	}
	// Selected tests explicitly check once all relevant work has completed.
	r.Check(t)
}

func TestSpecRejectsOverselling(t *testing.T) {
	t.Parallel()
	r := prstest.New()
	r.Emit("eStocked", "coffee", map[string]any{"units": 3})
	r.Emit("eReserved", "coffee", map[string]any{"order": "first", "units": 2})
	r.Emit("eReserved", "coffee", map[string]any{"order": "second", "units": 2})
	err := r.Verify(t)
	if err == nil || !strings.Contains(err.Error(), `"kind": "spec_violation"`) ||
		!strings.Contains(err.Error(), "reservation exceeds available stock") {
		t.Fatalf("want an overselling spec violation; got %v", err)
	}
}
