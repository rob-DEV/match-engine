use std::cmp::Ordering;
use std::time::{SystemTime, UNIX_EPOCH};

use common::message::{CancelOrder, OrderAction};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Order {
    New(LimitOrder),
    Cancel(CancelOrder),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct LimitOrder {
    pub id: u32,
    pub px: u32,
    pub qty: u32,
    pub side: OrderAction,
    pub placed_time: u128,
}

impl LimitOrder {
    pub fn new(id: u32, quantity: u32, price: u32, side: OrderAction) -> LimitOrder {
        LimitOrder {
            id,
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

impl PartialOrd for LimitOrder {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (&self.side, &other.side) {
            (&OrderAction::BUY, &OrderAction::BUY) => self.partial_cmp_buy(other),
            (&OrderAction::SELL, &OrderAction::SELL) => self.partial_cmp_sell(other),
            (_, _) => None,
        }
    }
}

impl Ord for LimitOrder {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}
