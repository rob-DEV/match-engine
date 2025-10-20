use crate::book::order_book::LimitOrderBook;
use common::domain::execution::Execution;
use common::domain::order::LimitOrder;

pub trait MatchStrategy {
    fn new() -> Self;
    fn match_orders(
        &mut self,
        order_book: &mut LimitOrderBook,
        order: &mut LimitOrder,
        mutable_execution_buffer: &mut [Execution],
    ) -> usize;
}
