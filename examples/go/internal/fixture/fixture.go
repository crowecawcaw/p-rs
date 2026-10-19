// Package fixture holds the inventory example's prstest configuration. Recording,
// checking and coverage come from github.com/crowecawcaw/p-rs/go/prstest.
package fixture

import (
	"testing"

	"example.com/inventory/inventory"
	"github.com/crowecawcaw/p-rs/go/prstest"
)

// Config is relative to each test package, one level below the module root.
func Config() prstest.Config {
	return prstest.Config{SpecFiles: []string{"../specs/inventory.p"}, Specs: []string{"InventoryCorrect"}}
}

// New returns a recorder that the test checks explicitly.
func New(t testing.TB) *prstest.Recorder {
	t.Helper()
	return prstest.New(t, Config())
}

// Checked returns an empty inventory whose events are checked at test cleanup.
func Checked(t testing.TB) *inventory.Inventory {
	t.Helper()
	return inventory.New(prstest.Start(t, Config()).Emit)
}

// CheckedWithCoverage is Checked, also adding the observation to suite.
func CheckedWithCoverage(t testing.TB, suite *prstest.Suite) *inventory.Inventory {
	t.Helper()
	cfg := Config()
	cfg.Suite = suite
	return inventory.New(prstest.Start(t, cfg).Emit)
}
