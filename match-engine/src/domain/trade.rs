use std::fmt::{Debug, Formatter};

use crate::domain::order::LimitOrder;

pub struct Trade {
    pub filled_quantity: u32,
    pub ask: LimitOrder,
    pub bid: LimitOrder,
}

impl Debug for Trade {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{0: <3} | {1: <5} | {2: <5} | {3: <4}",
            self.filled_quantity, self.bid.px, self.bid.id, self.ask.id
        )
    }
}
