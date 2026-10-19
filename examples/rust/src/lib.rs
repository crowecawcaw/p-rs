//! Application code: typed domain events, no dependency on P or a test framework.
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum Event {
    Stocked { sku: String, units: i64 },
    Reserved { sku: String, order: String, units: i64 },
    Released { sku: String, order: String, units: i64 },
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    UnknownProduct,
    AlreadyStocked,
    InvalidQuantity,
    DuplicateOrder,
    InsufficientStock,
    UnknownOrder,
}

#[derive(Default)]
struct Product {
    available: i64,
    reservations: HashMap<String, i64>,
}

pub struct Inventory<S> {
    products: HashMap<String, Product>,
    sink: S,
}

impl<S: FnMut(Event)> Inventory<S> {
    pub fn new(sink: S) -> Self {
        Self { products: HashMap::new(), sink }
    }

    pub fn stock(&mut self, sku: &str, units: i64) -> Result<(), Error> {
        if units < 0 {
            return Err(Error::InvalidQuantity);
        }
        if self.products.contains_key(sku) {
            return Err(Error::AlreadyStocked);
        }
        self.products.insert(sku.into(), Product { available: units, ..Default::default() });
        (self.sink)(Event::Stocked { sku: sku.into(), units });
        Ok(())
    }

    pub fn reserve(&mut self, sku: &str, order: &str, units: i64) -> Result<(), Error> {
        let product = self.products.get_mut(sku).ok_or(Error::UnknownProduct)?;
        if units <= 0 {
            return Err(Error::InvalidQuantity);
        }
        if product.reservations.contains_key(order) {
            return Err(Error::DuplicateOrder);
        }
        if product.available < units {
            return Err(Error::InsufficientStock);
        }
        product.available -= units;
        product.reservations.insert(order.into(), units);
        (self.sink)(Event::Reserved { sku: sku.into(), order: order.into(), units });
        Ok(())
    }

    pub fn release(&mut self, sku: &str, order: &str) -> Result<(), Error> {
        let product = self.products.get_mut(sku).ok_or(Error::UnknownProduct)?;
        let units = product.reservations.remove(order).ok_or(Error::UnknownOrder)?;
        product.available += units;
        (self.sink)(Event::Released { sku: sku.into(), order: order.into(), units });
        Ok(())
    }

    pub fn available(&self, sku: &str) -> Option<i64> {
        self.products.get(sku).map(|p| p.available)
    }
}
