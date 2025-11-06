use crate::domain::limit_order::LimitOrder;
use common::message::execution::ExecType;
use std::fmt::{Debug, Formatter};

pub struct Execution {
    pub id: u32,
    pub ask: LimitOrder,
    pub bid: LimitOrder,
    pub exec_qty: u32,
    pub exec_type: ExecType,
    pub execution_time: u64,
}

impl Debug for Execution {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "-----------------------------------Full Exec.-----------------------------------"
        )
        .unwrap();
        writeln!(
            f,
            "{0: <10} | {1: <10} | {2: <10} | {3: <10} | {4: <10}",
            "Ask id", "Bid", "Px", "Fill", "Ex Time"
        )
        .unwrap();
        writeln!(
            f,
            "{0: <10} | {1: <10} | {2: <10} | {3: <10} | {4: <10}",
            self.ask.id, self.bid.id, self.ask.px, self.exec_qty, self.execution_time
        )
        .unwrap();
        writeln!(
            f,
            "--------------------------------------------------------------------------------"
        )
        .unwrap();
        Ok(())
    }
}
