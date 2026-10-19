// Package fixture holds the inventory example's prstest configuration. Recording,
// checking and coverage come from github.com/crowecawcaw/p-rs/go/prstest.
package fixture

import (
	"testing"

	"example.com/inventory/inventory"
	"github.com/crowecawcaw/p-rs/go/prstest"
)

// Config is relative to each test package, one level below the module root.
// Naming the spec makes the check strict: it fails if no event reaches it.
func Config() prstest.Config {
	return prstest.Config{SpecFiles: []string{"../specs/inventory.p"}, Specs: []string{"InventoryCorrect"}}
}

// New returns a recorder that the test checks explicitly.
func New(t testing.TB) *prstest.Recorder {
	t.Helper()
	return prstest.New(t, Config())
}

// Start returns a recorder that is checked when the test ends.
func Start(t testing.TB) *prstest.Recorder {
	t.Helper()
	return prstest.Start(t, Config())
}

// Checked returns an empty inventory whose events are checked when the test ends.
func Checked(t testing.TB) *inventory.Inventory {
	t.Helper()
	return inventory.New(Start(t).Emit)
}

// CheckedWithCoverage is Checked, also adding the check's monitor coverage to
// coverage/monitor.json in the test package (written by prstest.Main).
func CheckedWithCoverage(t testing.TB) *inventory.Inventory {
	t.Helper()
	cfg := Config()
	cfg.Coverage = "coverage/monitor.json"
	return inventory.New(prstest.Start(t, cfg).Emit)
}
