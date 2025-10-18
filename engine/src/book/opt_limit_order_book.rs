use crate::book::book::Book;
use common::domain::domain::Side;
use common::domain::order::{CancelOrder, LimitOrder};
use std::collections::{BTreeMap, HashMap, LinkedList};

pub type Price = u32;
pub type LimitOrderList = LinkedList<LimitOrder>;

pub struct HalfBook {
    pub price_tree_map: BTreeMap<Price, LimitOrderList>,
    pub order_map: HashMap<u32, LimitOrder>,

    pub volume: u32,
    pub num_orders: u32,
}
impl HalfBook {
    pub fn new() -> Self {
        Self {
            price_tree_map: BTreeMap::new(),
            order_map: HashMap::with_capacity(1_000_000),
            volume: 0,
            num_orders: 0,
        }
    }

    pub fn add_price(&mut self, px: Price) {
        let order_list = LimitOrderList::new();
        self.price_tree_map.insert(px, order_list);
    }

    pub fn remove_price(&mut self, px: Price) {
        self.price_tree_map.remove(&px);
    }

    pub fn price_exists(&self, px: Price) -> bool {
        self.price_tree_map.contains_key(&px)
    }

    pub fn add_order(&mut self, order: LimitOrder) {
        let order_id = order.id;
        let order_px = order.px;
        let order_qty = order.qty;

        if !self.price_exists(order_px) {
            self.add_price(order_px);
        }

        self.price_tree_map
            .get_mut(&order_px)
            .unwrap()
            .push_back(order);
        self.order_map.insert(order_id, order);
        self.num_orders += 1;
        self.volume += order_qty;
    }

    pub fn modify_order(&mut self, id: u32, new_qty: u32) {
        let order = self.order_map.get_mut(&id);

        match order {
            Some(order) => {
                self.volume -= (order.qty - new_qty);
                order.qty = new_qty;

                if let Some(priceList) = self.price_tree_map.get_mut(&order.px) {
                    for price_tree_order in priceList {
                        if price_tree_order.id == id {
                            price_tree_order.qty = new_qty;
                            break;
                        }
                    }
                }
            }
            None => {}
        }
    }

    pub fn remove_order(&mut self, id: u32) {
        let order = self.order_map.get(&id);

        match order {
            Some(order) => {
                let order_px = order.px;
                self.volume -= order.qty;
                self.num_orders -= 1;

                if self.price_tree_map.get(&order_px).is_none() {
                    self.remove_price(order_px);
                }

                self.price_tree_map.remove(&order_px);
                self.order_map.remove(&id);
            }
            None => {}
        }
    }

    pub fn volume(&self) -> u32 {
        self.volume
    }
}
pub struct OptLimitOrderBook {
    pub asks: HalfBook,
    pub bids: HalfBook,
}

impl OptLimitOrderBook {
    pub fn new() -> Self {
        Self {
            asks: HalfBook::new(),
            bids: HalfBook::new(),
        }
    }
}

impl Book for OptLimitOrderBook {
    fn add_order(&mut self, order: LimitOrder) {
        match order.action {
            Side::BUY => {
                self.bids.add_order(order);
            }
            Side::SELL => {
                self.asks.add_order(order);
            }
        };
    }

    fn modify_order(&mut self, action: Side, order_id: u32, new_qty: u32) {
        match action {
            Side::BUY => {
                self.bids.modify_order(order_id, new_qty);
            }
            Side::SELL => {
                self.asks.modify_order(order_id, new_qty);
            }
        };
    }

    fn remove_order(&mut self, order: CancelOrder) -> bool {
        match order.action {
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
