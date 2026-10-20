package checked_test

import (
	"testing"

	"example.com/inventory/internal/fixture"
	"example.com/inventory/inventory"
	"github.com/crowecawcaw/snappea/go/snappeatest"
)

// stocked starts recording, registers the cleanup check, then seeds stock.
// Every participating test must call it: Go has no global per-test hook.
func stocked(t *testing.T) *inventory.Inventory {
	t.Helper()
	app := fixture.CheckedWithCoverage(t)
	stock := map[string]int{"coffee": 3, "tea": 2}
	for _, sku := range []string{"coffee", "tea"} {
		if err := app.Stock(sku, stock[sku]); err != nil {
			t.Fatal(err)
		}
	}
	return app
}

func TestIndependentProducts(t *testing.T) {
	t.Parallel()
	app := stocked(t)
	for _, sku := range []string{"coffee", "tea"} {
		if err := app.Reserve(sku, "order-1", 2); err != nil {
			t.Fatal(err)
		}
	}
	if err := app.Release("tea", "order-1"); err != nil {
		t.Fatal(err)
	}
	if app.Available("coffee") != 1 || app.Available("tea") != 2 {
		t.Fatal("incorrect product balance")
	}
}

func TestRejectedReservation(t *testing.T) {
	t.Parallel()
	app := stocked(t)
	if err := app.Reserve("coffee", "order-1", 2); err != nil {
		t.Fatal(err)
	}
	if err := app.Reserve("coffee", "order-2", 2); err == nil {
		t.Fatal("expected insufficient stock")
	}
	if app.Available("coffee") != 1 {
		t.Fatal("rejected operation changed stock")
	}
}

// TestMain runs the tests, then writes coverage/monitor.json and prints its summary.
func TestMain(m *testing.M) { snappeatest.Main(m) }
