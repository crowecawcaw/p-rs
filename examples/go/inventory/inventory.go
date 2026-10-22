// Package inventory implements an in-memory inventory service. Tests pass a
// recording's Emit; programs pass snappeaemit.Emit (see cmd/inventory).
package inventory

import "errors"

// Emit receives a committed change. A nil Emit disables observation.
type Emit func(event, key string, payload any)

type product struct {
	available int
	orders    map[string]int
}

type Inventory struct {
	products map[string]*product
	emit     Emit
}

func New(emit Emit) *Inventory {
	if emit == nil {
		emit = func(string, string, any) {}
	}
	return &Inventory{products: make(map[string]*product), emit: emit}
}

func (i *Inventory) Stock(sku string, units int) error {
	if units < 0 || i.products[sku] != nil {
		return errors.New("invalid stock or duplicate product")
	}
	i.products[sku] = &product{available: units, orders: make(map[string]int)}
	i.emit("eStocked", sku, map[string]any{"units": units})
	return nil
}

func (i *Inventory) Reserve(sku, order string, units int) error {
	p := i.products[sku]
	if p == nil || units <= 0 || units > p.available {
		return errors.New("invalid reservation or insufficient stock")
	}
	if _, exists := p.orders[order]; exists {
		return errors.New("duplicate order")
	}
	p.available -= units
	p.orders[order] = units
	i.emit("eReserved", sku, map[string]any{"order": order, "units": units})
	return nil
}

func (i *Inventory) Release(sku, order string) error {
	p := i.products[sku]
	if p == nil {
		return errors.New("unknown product")
	}
	units, exists := p.orders[order]
	if !exists {
		return errors.New("unknown reservation")
	}
	delete(p.orders, order)
	p.available += units
	i.emit("eReleased", sku, map[string]any{"order": order, "units": units})
	return nil
}

func (i *Inventory) Available(sku string) int { return i.products[sku].available }
