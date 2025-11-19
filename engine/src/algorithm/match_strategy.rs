use crate::book::order_book::LimitOrderBook;
use crate::domain::order::LimitOrder;
use common::types::execution_report::ExecutionReport;

pub trait MatchStrategy: std::fmt::Debug {
    fn match_orders(
        &mut self,
        order_book: &mut LimitOrderBook,
        order: &mut LimitOrder,
        mutable_execution_buffer: &mut Vec<ExecutionReport>,
    ) -> usize;
}
