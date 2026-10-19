package checked_test

import (
	"fmt"
	"os"
	"testing"

	"example.com/inventory/internal/prstest"
	"example.com/inventory/inventory"
)

// The project fixture opts each test into cleanup checking. Go has no global
// per-test hook: TestMain alone cannot give every test an isolated recorder.
func stocked(t *testing.T) *inventory.Inventory {
	t.Helper()
	app := prstest.CheckedWithCoverage(t)
	for _, sku := range []string{"coffee", "tea"} {
		if err := app.Stock(sku, 3); err != nil {
			t.Fatal(err)
		}
	}
	return app
}

func TestIndependentProducts(t *testing.T) {
	t.Parallel()
	app := stocked(t)
	for _, sku := range []string{"coffee", "tea"} {
		if err := app.Reserve(sku, "same-order", 2); err != nil {
			t.Fatal(err)
		}
	}
	if err := app.Release("tea", "same-order"); err != nil {
		t.Fatal(err)
	}
	if app.Available("coffee") != 1 || app.Available("tea") != 3 {
		t.Fatal("incorrect product balance")
	}
}

func TestRejectedReservation(t *testing.T) {
	t.Parallel()
	app := stocked(t)
	if err := app.Reserve("coffee", "first", 2); err != nil {
		t.Fatal(err)
	}
	if err := app.Reserve("coffee", "second", 2); err == nil {
		t.Fatal("expected insufficient stock")
	}
	if app.Available("coffee") != 1 {
		t.Fatal("rejected operation changed stock")
	}
}

func TestMain(m *testing.M) {
	code := m.Run()
	if err := prstest.WriteCoverage("coverage/model.json"); err != nil {
		fmt.Fprintln(os.Stderr, err)
		code = 1
	}
	os.Exit(code)
}
