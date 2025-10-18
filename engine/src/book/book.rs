use common::domain::domain::Side;
use common::domain::order::{CancelOrder, LimitOrder};

pub trait Book {
    fn add_order(&mut self, order: LimitOrder);
    fn modify_order(&mut self, action: Side, order_id: u32, new_qty: u32);
    fn remove_order(&mut self, order_id: CancelOrder) -> bool;
    fn orders_on_book(&mut self) -> usize;
    fn bid_volume(&self) -> u32;
    fn ask_volume(&self) -> u32;
    fn total_volume(&self) -> u32;
}
