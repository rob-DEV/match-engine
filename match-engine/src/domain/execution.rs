use std::fmt::{Debug, Formatter};

use crate::domain::order::LimitOrder;

pub enum Execution {
    FullMatch(FullMatch),
    PartialMatch(PartialMatch),
}

pub struct FullMatch {
    pub id: u32,
    pub ask: LimitOrder,
    pub bid: LimitOrder,
    pub execution_time: u128,
}

pub struct PartialMatch {
    pub id: u32,
    pub fill_qty: u32,
    pub ask: LimitOrder,
    pub bid: LimitOrder,
    pub execution_time: u128,
}

impl Debug for FullMatch {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "-----------------------------------Full Exec.-----------------------------------").unwrap();
        writeln!(
            f,
            "{0: <10} | {1: <10} | {2: <10} | {3: <10}",
            "Ask id", "Bid", "Px", "Ex Time"
        ).unwrap();
        writeln!(
            f,
            "{0: <10} | {1: <10} | {2: <10} | {3: <10}",
            self.ask.id, self.bid.id, self.ask.px, self.execution_time
        ).unwrap();
        writeln!(f, "--------------------------------------------------------------------------------").unwrap();
        Ok(())
    }
}

impl Debug for PartialMatch {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "-----------------------------------Partial Exec.-----------------------------------").unwrap();
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
        writeln!(f, "-----------------------------------------------------------------------------------").unwrap();
        Ok(())
    }
}
