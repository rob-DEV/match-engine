use crate::book::book::Book;
use common::domain::execution::Execution;
use common::util::time::epoch_nanos;
use rand::random;

use crate::algorithm::match_strategy::MatchStrategy;
use crate::book::opt_limit_order_book::OptLimitOrderBook;

pub struct FifoMatchStrategy;

impl MatchStrategy for FifoMatchStrategy {
    fn match_orders(
        &self,
        order_book: &mut OptLimitOrderBook,
        mutable_execution_buffer: &mut [Execution],
    ) -> usize {
        let mut num_executions: usize = 0;
        
        num_executions
    }
}
