package checked_test

import (
	"context"
	"fmt"
	"os"
	"testing"

	"example.com/inventory/internal/prstest"
	"example.com/inventory/inventory"
	helper "github.com/crowecawcaw/p-rs/go/prstest"
)

var suite *helper.Suite

// The project fixture opts each test into cleanup checking. Go has no global
// per-test hook: TestMain alone cannot give every test an isolated recorder.
func stocked(t *testing.T) *inventory.Inventory {
	t.Helper()
	app := prstest.CheckedWithCoverage(t, suite)
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
	var err error
	suite, err = helper.NewSuite(helper.SuiteConfig{})
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	code := m.Run()
	if err := suite.WriteCoverage(context.Background(), "coverage/model.json", os.Stdout); err != nil {
		fmt.Fprintln(os.Stderr, err)
		code = 1
	}
	os.Exit(code)
}
