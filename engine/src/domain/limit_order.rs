use common::types::new_order::TimeInForce;
use common::types::side::Side;
use std::cmp::Ordering;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct LimitOrder {
    pub client_id: u32,
    pub id: u32,
    pub side: Side,
    pub px: u32,
    pub qty: u32,
    pub time_in_force: TimeInForce,
    pub placed_time: u64,
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
        match (&self.side, &other.side) {
            (&Side::BUY, &Side::BUY) => self.partial_cmp_buy(other),
            (&Side::SELL, &Side::SELL) => self.partial_cmp_sell(other),
            (_, _) => None,
        }
    }
}

impl Ord for LimitOrder {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}
