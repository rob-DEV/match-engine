use crate::book::opt_limit_order_book::OptLimitOrderBook;
use common::domain::execution::Execution;

pub trait MatchStrategy {
    fn match_orders(
        &self,
        order_book: &mut OptLimitOrderBook,
        mutable_execution_buffer: &mut [Execution],
    ) -> usize;
}
