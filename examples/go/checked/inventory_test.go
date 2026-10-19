package checked_test

import (
	"context"
	"fmt"
	"os"
	"testing"

	"example.com/inventory/internal/fixture"
	"example.com/inventory/inventory"
	"github.com/crowecawcaw/p-rs/go/prstest"
)

var suite *prstest.Suite

// stocked starts recording, registers the cleanup check, then seeds stock.
// Every participating test must call it: Go has no global per-test hook.
func stocked(t *testing.T) *inventory.Inventory {
	t.Helper()
	app := fixture.CheckedWithCoverage(t, suite)
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

func TestMain(m *testing.M) {
	var err error
	suite, err = prstest.NewSuite(prstest.SuiteConfig{})
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	code := m.Run()
	if err := suite.WriteCoverage(context.Background(), "coverage/monitor.json", os.Stdout); err != nil {
		fmt.Fprintln(os.Stderr, err)
		code = 1
	}
	os.Exit(code)
}
