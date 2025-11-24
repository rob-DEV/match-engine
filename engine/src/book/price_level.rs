use std::collections::VecDeque;

#[derive(Debug, Default)]
pub struct PriceLevel {
    order_ids: VecDeque<u32>,
    pub total_qty: u32,
    pub num_orders: u32,
}

impl PriceLevel {
    pub fn new() -> PriceLevel {
        PriceLevel {
            order_ids: VecDeque::with_capacity(100_000),
            total_qty: 0,
            num_orders: 0,
        }
    }

    pub fn add_order(&mut self, order_id: u32, qty: u32) {
        self.order_ids.push_back(order_id);
        self.total_qty += qty;
        self.num_orders += 1;
    }

    pub fn adjust_qty(&mut self, qty: u32) {
        self.total_qty = self.total_qty.saturating_sub(qty);
    }

    pub fn remove_order(&mut self, order_id: u32, qty: u32) {
        if let Some(pos) = self.order_ids.iter().position(|&id| id == order_id) {
            self.order_ids.remove(pos);
            self.total_qty = self.total_qty.saturating_sub(qty);
            self.num_orders = self.num_orders.saturating_sub(1);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.order_ids.is_empty()
    }

    pub fn front(&self) -> Option<&u32> {
        self.order_ids.front()
    }

    pub fn pop_front(&mut self) -> Option<u32> {
        self.order_ids.pop_front()
    }

    pub fn order_ids(&self) -> impl Iterator<Item = &u32> {
        self.order_ids.iter()
    }
    
    pub fn volume(&self) -> u32 {
        self.total_qty
    }

    pub fn num_orders(&self) -> u32 {
        self.num_orders
    }
}
