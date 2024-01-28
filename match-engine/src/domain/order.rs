use std::cmp::Ordering;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::domain::side::OrderAction;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OrderType {
    New,
    Cancel,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Order {
    pub id: u32,
    pub order_type: OrderType,
    pub px: u32,
    pub qty: u32,
    pub side: OrderAction,
    pub placed_time: u128,
}

impl Order {
    pub fn new(id: u32, order_type: OrderType, quantity: u32, price: u32, side: OrderAction) -> Order {
        Order {
            id,
            order_type,
            qty: quantity,
            px: price,
            side,
            placed_time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos(),
        }
    }

    fn partial_cmp_buy(&self, other: &Self) -> Option<Ordering> {
        Some(self.px.cmp(&other.px))
    }

    fn partial_cmp_sell(&self, other: &Self) -> Option<Ordering> {
        Some(other.px.cmp(&self.px))
    }
}

impl PartialOrd for Order {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (&self.side, &other.side) {
            (&OrderAction::BUY, &OrderAction::BUY) => self.partial_cmp_buy(other),
            (&OrderAction::SELL, &OrderAction::SELL) => self.partial_cmp_sell(other),
            (_, _) => None,
        }
    }
}

impl Ord for Order {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}
