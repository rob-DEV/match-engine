use crate::book::book::Book;
use common::domain::domain::Side;
use common::domain::domain::Side::{BUY, SELL};
use common::domain::order::{CancelOrder, LimitOrder};
use std::collections::{BTreeMap, HashMap, VecDeque};

pub type Price = u32;
pub type LimitOrderList = Vec<LimitOrder>;

pub struct BookSide {
    pub price_level_map: BTreeMap<Price, VecDeque<u32>>,
    pub order_map: HashMap<u32, LimitOrder>,

    pub side: Side,
    pub volume: u32,
    pub num_orders: u32,
}
impl BookSide {
    pub fn new(side: Side) -> Self {
        Self {
            price_level_map: BTreeMap::new(),
            order_map: HashMap::with_capacity(1_000_000),
            side,
            volume: 0,
            num_orders: 0,
        }
    }

    pub fn add_order(&mut self, order: LimitOrder) {
        self.price_level_map
            .entry(order.px)
            .or_default()
            .push_back(order.id); // FIFO: push to back
        self.order_map.insert(order.id, order);

        self.num_orders += 1;
        self.volume += order.qty;
    }

    pub fn modify_order(&mut self, id: u32, new_qty: u32) {
        if let Some(order) = self.order_map.get_mut(&id) {
            self.volume = self.volume - order.qty + new_qty;
            order.qty = new_qty;
        }
    }

    pub fn remove_order(&mut self, id: u32) {
        if let Some(order) = self.order_map.remove(&id) {
            if let Some(level) = self.price_level_map.get_mut(&order.px) {
                level.retain(|&x| x != id);
                if level.is_empty() {
                    self.price_level_map.remove(&order.px);
                }
            }
            self.volume -= order.qty;
            self.num_orders -= 1;
        }
    }

    pub fn best_price(&self) -> Option<Price> {
        match self.side {
            Side::BUY => self.price_level_map.keys().next_back().copied(),
            Side::SELL => self.price_level_map.keys().next().copied(),
        }
    }

    pub fn volume(&self) -> u32 {
        self.volume
    }
}
pub struct OptLimitOrderBook {
    pub asks: BookSide,
    pub bids: BookSide,
}

impl OptLimitOrderBook {
    pub fn new() -> Self {
        Self {
            asks: BookSide::new(SELL),
            bids: BookSide::new(BUY),
        }
    }
}

impl Book for OptLimitOrderBook {
    fn add_order(&mut self, order: LimitOrder) {
        match order.side {
            Side::BUY => {
                self.bids.add_order(order);
            }
            Side::SELL => {
                self.asks.add_order(order);
            }
        };
    }

    fn modify_order(&mut self, side: Side, order_id: u32, new_qty: u32) {
        match side {
            Side::BUY => {
                self.bids.modify_order(order_id, new_qty);
            }
            Side::SELL => {
                self.asks.modify_order(order_id, new_qty);
            }
        };
    }

    fn remove_order(&mut self, order: CancelOrder) -> bool {
        match order.side {
            Side::BUY => {
                self.bids.remove_order(order.id);
            }
            Side::SELL => {
                self.asks.remove_order(order.id);
            }
        };

        true
    }

    fn orders_on_book(&mut self) -> usize {
        (self.asks.num_orders + self.bids.num_orders) as usize
    }

    fn bid_volume(&self) -> u32 {
        self.bids.volume()
    }

    fn ask_volume(&self) -> u32 {
        self.asks.volume()
    }

    fn total_volume(&self) -> u32 {
        self.bid_volume() + self.ask_volume()
    }
}
