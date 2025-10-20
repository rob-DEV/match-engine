use crate::algorithm::match_strategy::MatchStrategy;
use crate::book::order_book::LimitOrderBook;
use common::domain::execution::Execution;
use common::domain::order::LimitOrder;

pub struct ProRataMatchStrategy;

impl MatchStrategy for ProRataMatchStrategy {
    fn new() -> Self {
        todo!()
    }

    fn match_orders(
        &mut self,
        order_book: &mut LimitOrderBook,
        order: &mut LimitOrder,
        mutable_execution_buffer: &mut [Execution],
    ) -> usize {
        let mut num_executions: usize = 0;

        num_executions
    }
}

impl ProRataMatchStrategy {}
