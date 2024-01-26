use std::fmt::{Debug, Formatter};

use crate::domain::order::Order;

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
            self.filled_quantity, self.bid.price, self.bid.identifier, self.ask.identifier
        )
    }
}
