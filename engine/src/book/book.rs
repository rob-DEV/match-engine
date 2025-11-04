use common::message::cancel_order::CancelOrder;
use crate::domain::order::LimitOrder;

pub trait Book {
    fn add_order(&mut self, order: LimitOrder);
    fn remove_order(&mut self, cancel_order: &CancelOrder) -> bool;
    fn orders_on_book(&mut self) -> usize;
    fn bid_volume(&self) -> u32;
    fn ask_volume(&self) -> u32;
    fn total_volume(&self) -> u32;
}
