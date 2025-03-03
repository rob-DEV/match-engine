use crate::internal::execution::Execution;
use crate::internal::order::{CancelOrder, LimitOrder};

pub trait Book {
    fn apply(&mut self, order: LimitOrder);
    fn check_for_trades(&mut self, max_execution_per_cycle: usize, arr: &mut [Execution]) -> usize;
    fn cancel(&mut self, order_id: CancelOrder) -> bool;

    fn count_resting_orders(&mut self) -> usize;
}