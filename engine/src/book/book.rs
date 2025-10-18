use common::domain::execution::Execution;
use common::domain::order::{CancelOrder, LimitOrder};

pub trait Book {
    fn add_order(&mut self, order: LimitOrder);
    // fn match_orders(&mut self, max_execution_per_cycle: usize, arr: &mut [Execution]) -> usize;
    fn remove_order(&mut self, order_id: CancelOrder) -> bool;
    fn orders_on_book(&mut self) -> usize;
    fn bid_volume(&self) -> u32;
    fn ask_volume(&self) -> u32;
    fn total_volume(&self) -> u32;
}