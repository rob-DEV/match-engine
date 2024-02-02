use std::fmt::{Debug, Formatter};

use crate::domain::order::LimitOrder;

pub struct Execution {
    pub id: u32,
    pub fill_qty: u32,
    pub ask: LimitOrder,
    pub bid: LimitOrder,
    pub execution_time: u128,
}

impl Debug for Execution {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "-----------------------------------Execution-----------------------------------").unwrap();
        writeln!(
            f,
            "{0: <8} | {1: <10} | {2: <10} | {3: <10} | {4: <10}",
            "Fill Qty", "Ask id", "Bid", "Px", "Ex Time"
        ).unwrap();
        writeln!(
            f,
            "{0: <8} | {1: <10} | {2: <10} | {3: <10} | {4: <10}",
            self.fill_qty, self.ask.id, self.bid.id, self.ask.px, self.execution_time
        ).unwrap();
        writeln!(f, "-----------------------------------Execution-----------------------------------").unwrap();
        Ok(())
    }
}
