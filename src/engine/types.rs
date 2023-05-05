use std::{cmp::Ordering, fmt::Debug, fmt::Formatter};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Side {
    BUY,
    SELL,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Order {
    pub client_id: u64,
    pub seq_number: u64,
    pub price: u64,
    pub quantity: u64,
    pub side: Side,
}

impl Order {
    pub fn new(client_id: u64, seq_number: u64, price: u64, quantity: u64, side: Side) -> Order {
        Order {
            client_id,
            seq_number,
            price,
            quantity,
            side,
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
        self.partial_cmp(other).unwrap_or(Ordering::Equal) // Sell and Buy are non-comparable
    }
}

pub struct Trade {
    pub filled_quantity: u64,
    pub ask: Order,
    pub bid: Order,
}

impl Debug for Trade {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{0: <3} | {1: <5} | {2: <5} | {3: <4}",
            self.filled_quantity, self.bid.price, self.bid.client_id, self.ask.client_id
        )
    }
}
