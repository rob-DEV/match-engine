use std::{cmp::Ordering, fmt::Debug, fmt::Formatter};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Side {
    BUY,
    SELL,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Order {
    pub client_id: u32,
    pub seq_number: u32,
    pub price: u32,
    pub quantity: u32,
    pub side: Side,
}

impl Order {
    pub fn new(client_id: u32, seq_number: u32, quantity: u32, price: u32, side: Side) -> Order {
        Order {
            client_id,
            seq_number,
            quantity,
            price,
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
    pub filled_quantity: u32,
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
