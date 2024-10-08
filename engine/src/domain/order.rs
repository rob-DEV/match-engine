use std::cmp::Ordering;

use common::engine::OrderAction;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Order {
    New(LimitOrder),
    Cancel(CancelOrder),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct LimitOrder {
    pub client_id: u32,
    pub id: u32,
    pub action: OrderAction,
    pub px: u32,
    pub qty: u32,
    pub placed_time: u64,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CancelOrder {
    pub client_id: u32,
    pub action: OrderAction,
    pub id: u32,
}

impl LimitOrder {
    fn partial_cmp_buy(&self, other: &Self) -> Option<Ordering> {
        Some(self.px.cmp(&other.px))
    }

    fn partial_cmp_sell(&self, other: &Self) -> Option<Ordering> {
        Some(other.px.cmp(&self.px))
    }
}

impl PartialOrd for LimitOrder {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (&self.action, &other.action) {
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
