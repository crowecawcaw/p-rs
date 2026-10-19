// Package prstest supplies only inventory-specific configuration and fixtures.
// Recording, CLI execution, strict validation, and coverage are supported by the
// repository's Go helper module.
package prstest

import (
	"testing"

	"example.com/inventory/inventory"
	helper "github.com/crowecawcaw/p-rs/go/prstest"
)

// Go runs each package from its own directory. All example test packages are one
// level below the module root; specs live in the shared Python example directory.
func Config() helper.Config {
	return helper.Config{SpecFiles: []string{"../../python/specs/inventory.p"}, Specs: []string{"InventoryCorrect"}}
}

func New(t testing.TB) *helper.Recorder { t.Helper(); return helper.New(t, Config()) }

func Checked(t testing.TB) *inventory.Inventory {
	t.Helper()
	return inventory.New(helper.Start(t, Config()).Emit)
}

func CheckedWithCoverage(t testing.TB, suite *helper.Suite) *inventory.Inventory {
	t.Helper()
	cfg := Config()
	cfg.Suite = suite
	return inventory.New(helper.Start(t, cfg).Emit)
}
