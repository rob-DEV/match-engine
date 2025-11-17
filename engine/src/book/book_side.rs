use crate::book::order_book::Price;
use crate::book::price_level::PriceLevel;
use crate::domain::order::LimitOrder;
use common::types::side::Side;
use common::types::side::Side::{BUY, SELL};
use std::collections::{BTreeMap, HashMap};

pub struct BookSide {
    pub price_level_map: BTreeMap<Price, PriceLevel>,
    pub order_map: HashMap<u32, LimitOrder>,
    pub side: Side,

    pub total_qty: u32,
    pub num_orders: u32,
}
impl BookSide {
    pub fn new(side: Side) -> Self {
        Self {
            price_level_map: BTreeMap::new(),
            order_map: HashMap::with_capacity(1_000_000),
            side,
            total_qty: 0,
            num_orders: 0,
        }
    }

    pub fn add_order(&mut self, order: LimitOrder) {
        self.price_level_map
            .entry(order.px)
            .or_default()
            .add_order(order.id, order.qty);

        self.order_map.insert(order.id, order);

        self.total_qty += order.qty;
        self.num_orders += 1;
    }

    pub fn remove_order(&mut self, id: u32) {
        if let Some(order) = self.order_map.remove(&id) {
            if let Some(level) = self.price_level_map.get_mut(&order.px) {
                level.remove_order(order.id, order.qty);
                self.total_qty -= order.qty;
                self.num_orders -= 1;
            }
        }
    }

    pub fn best_price(&self) -> Option<Price> {
        match self.side {
            BUY => self.price_level_map.keys().next_back().copied(),
            SELL => self.price_level_map.keys().next().copied(),
        }
    }

    pub fn num_orders(&self) -> u32 {
        self.num_orders
    }

    pub fn volume(&self) -> u32 {
        self.total_qty
    }
}
