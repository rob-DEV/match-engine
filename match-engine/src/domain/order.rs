use std::cmp::Ordering;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::domain::side::Side;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Order {
    pub id: u32,
    pub price: u32,
    pub quantity: u32,
    pub side: Side,
    pub placed_time: u128,
}

impl Order {
    pub fn new(id: u32, quantity: u32, price: u32, side: Side) -> Order {
        Order {
            id,
            quantity,
            price,
            side,
            placed_time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos(),
        }
    }

    fn partial_cmp_buy(&self, other: &Self) -> Option<Ordering> {
        Some(self.price.cmp(&other.price))
    }

    fn partial_cmp_sell(&self, other: &Self) -> Option<Ordering> {
        Some(other.price.cmp(&self.price))
    }
}

impl PartialOrd for Order {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (&self.side, &other.side) {
            (&Side::BUY, &Side::BUY) => self.partial_cmp_buy(other),
            (&Side::SELL, &Side::SELL) => self.partial_cmp_sell(other),
            (_, _) => None,
        }
    }
}

impl Ord for Order {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}
