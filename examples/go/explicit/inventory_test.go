package explicit_test

import (
	"context"
	"encoding/json"
	"strings"
	"testing"

	"example.com/inventory/internal/fixture"
	"example.com/inventory/inventory"
)

func TestReservationAndCancellation(t *testing.T) {
	t.Parallel()
	r := fixture.New(t)
	app := inventory.New(r.Emit) // start recording before stocking
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
	r.Check(t)
}

// A negative control: the spec must reject an overselling trace.
func TestSpecRejectsOverselling(t *testing.T) {
	t.Parallel()
	r := fixture.New(t)
	r.Emit("eStocked", "coffee", map[string]any{"units": 3})
	r.Emit("eReserved", "coffee", map[string]any{"order": "order-1", "units": 2})
	r.Emit("eReserved", "coffee", map[string]any{"order": "order-2", "units": 2})
	result, err := r.Verify(context.Background())
	if err == nil {
		t.Fatal("want an overselling spec violation; the check passed")
	}
	var report struct {
		Specs []struct{ Errors []struct{ Kind, Message string } }
	}
	if err := json.Unmarshal(result.Report, &report); err != nil || len(report.Specs) != 1 || len(report.Specs[0].Errors) == 0 {
		t.Fatalf("want a report with one spec's errors; got %v\n%s", err, result.Report)
	}
	if e := report.Specs[0].Errors[0]; e.Kind != "spec_violation" || !strings.Contains(e.Message, "reservation exceeds available stock") {
		t.Fatalf("want an overselling spec violation; got %+v", e)
	}
}
