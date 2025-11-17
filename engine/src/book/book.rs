use common::types::cancel_order::CancelOrderRequest;
use crate::domain::order::LimitOrder;

pub trait Book {
    fn add_order(&mut self, order: LimitOrder);
    fn remove_order(&mut self, cancel_order: &CancelOrderRequest) -> bool;
    fn orders_on_book(&mut self) -> usize;
    fn bid_volume(&self) -> u32;
    fn ask_volume(&self) -> u32;
    fn total_volume(&self) -> u32;
}
