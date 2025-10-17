use crate::book::book::Book;
use common::domain::domain::Side;
use common::domain::execution::Execution;
use common::domain::order::{CancelOrder, LimitOrder};
use common::util::time::epoch_nanos;
use rand::random;
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap, LinkedList};

type Price = u32;
type LimitOrderList = LinkedList<LimitOrder>;

pub struct HalfBook {
    pub price_tree_map: BTreeMap<Price, LimitOrderList>,
    pub order_map: HashMap<u32, LimitOrder>,

    pub volume: u32,
    pub depth: u32,
    pub num_orders: u32,
}
impl HalfBook {
    pub fn new() -> Self {
        Self {
            price_tree_map: BTreeMap::new(),
            order_map: HashMap::with_capacity(1_000_000),
            volume: 0,
            depth: 0,
            num_orders: 0,
        }
    }

    pub fn order_list_at_price(&self, px: Price) -> &LimitOrderList {
        &self.price_tree_map[&px]
    }

    pub fn order(&self, id: u32) -> &LimitOrder {
        &self.order_map[&id]
    }

    pub fn add_price(&mut self, px: Price) {
        self.depth += 1;
        let order_list = LimitOrderList::new();
        self.price_tree_map.insert(px, order_list);
    }

    pub fn remove_price(&mut self, px: Price) {
        self.depth -= 1;
        self.price_tree_map.remove(&px);
    }

    pub fn price_exists(&self, px: Price) -> bool {
        self.price_tree_map.contains_key(&px)
    }

    pub fn order_exists(&self, id: u32) -> bool {
        self.order_map.contains_key(&id)
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

    pub fn remove_order(&mut self, id: u32) {
        let order = self.order_map.get(&id).unwrap();
        let order_px = order.px;
        self.volume -= order.qty;
        self.num_orders -= 1;

        if self.price_tree_map.get(&order_px).is_none() {
            self.remove_price(order_px);
        }

        self.price_tree_map.remove(&order_px);
        self.order_map.remove(&id);
    }
}
pub struct OptLimitOrderBook {
    asks: HalfBook,
    bids: HalfBook,
}

impl OptLimitOrderBook {
    pub fn new() -> Self {
        Self {
            asks: HalfBook::new(),
            bids: HalfBook::new(),
        }
    }
}

fn attempt_order_match(
    ask: &LimitOrder,
    bid: &LimitOrder,
) -> Option<(Execution, Option<LimitOrder>)> {
    let (ask, bid) = match (ask.action, bid.action) {
        (Side::BUY, Side::SELL) => (bid, ask),
        (Side::SELL, Side::BUY) => (ask, bid),
        (_, _) => return None,
    };

    match ask.qty.cmp(&bid.qty) {
        Ordering::Equal => Some((
            Execution {
                id: random::<u32>(),
                fill_qty: ask.qty,
                ask: ask.clone(),
                bid: bid.clone(),
                execution_time: epoch_nanos(),
            },
            None,
        )),
        Ordering::Greater => {
            let quantity = bid.qty;
            let mut remainder = ask.clone();
            remainder.qty -= quantity;

            Some((
                Execution {
                    id: random::<u32>(),
                    fill_qty: quantity,
                    ask: ask.clone(),
                    bid: bid.clone(),
                    execution_time: epoch_nanos(),
                },
                Some(remainder),
            ))
        }
        Ordering::Less => {
            let quantity = ask.qty;
            let mut remainder = bid.clone();
            remainder.qty -= quantity;
            Some((
                Execution {
                    id: random::<u32>(),
                    fill_qty: quantity,
                    ask: ask.clone(),
                    bid: bid.clone(),
                    execution_time: epoch_nanos(),
                },
                Some(remainder),
            ))
        }
    }
}

impl Book for OptLimitOrderBook {
    fn apply(&mut self, order: LimitOrder) {
        match order.action {
            Side::BUY => {
                self.bids.add_order(order);
            }
            Side::SELL => {
                self.asks.add_order(order);
            }
        };
    }

    fn check_for_trades(&mut self, max_execution_per_cycle: usize, arr: &mut [Execution]) -> usize {
        let mut num_executions: usize = 0;

        while let (Some(mut ask_order_list_entry), Some(mut bid_order_list_entry)) = (
            self.asks.price_tree_map.first_entry(),
            self.bids.price_tree_map.last_entry(),
        ) {
            let ask_price = ask_order_list_entry.key();
            let bid_price = bid_order_list_entry.key();
            //

            if ask_price > bid_price {
                return num_executions;
            }

            let order_result: Vec<(u32, u32, Option<LimitOrder>)> = ask_order_list_entry
                .get()
                .iter()
                .zip(bid_order_list_entry.get().iter())
                // .take_while(|_| num_executions > max_execution_per_cycle)
                .map(|(ask, bid)| {
                    let (ask, bid) = match (ask.action, bid.action) {
                        (Side::BUY, Side::SELL) => (bid, ask),
                        (Side::SELL, Side::BUY) => (ask, bid),
                        (_, _) => return None,
                    };

                    if ask.px > bid.px {
                        return None;
                    }

                    let b = match ask.qty.cmp(&bid.qty) {
                        Ordering::Equal => Some((
                            Execution {
                                id: random::<u32>(),
                                fill_qty: ask.qty,
                                ask: ask.clone(),
                                bid: bid.clone(),
                                execution_time: epoch_nanos(),
                            },
                            None,
                        )),
                        Ordering::Greater => {
                            let quantity = bid.qty;
                            let mut remainder = ask.clone();
                            remainder.qty -= quantity;

                            Some((
                                Execution {
                                    id: random::<u32>(),
                                    fill_qty: quantity,
                                    ask: ask.clone(),
                                    bid: bid.clone(),
                                    execution_time: epoch_nanos(),
                                },
                                Some(remainder),
                            ))
                        }
                        Ordering::Less => {
                            let quantity = ask.qty;
                            let mut remainder = bid.clone();
                            remainder.qty -= quantity;
                            Some((
                                Execution {
                                    id: random::<u32>(),
                                    fill_qty: quantity,
                                    ask: ask.clone(),
                                    bid: bid.clone(),
                                    execution_time: epoch_nanos(),
                                },
                                Some(remainder),
                            ))
                        }
                    };

                    match b {
                        None => None,
                        Some((execution, rem)) => {
                            arr[num_executions] = execution;
                            num_executions += 1;
                            Some((ask.id, bid.id, rem))
                        }
                    }
                })
                .filter(|x| x.is_some())
                .map(|x| x.unwrap())
                .collect::<Vec<_>>();

            ask_order_list_entry.get_mut().pop_front();
            bid_order_list_entry.get_mut().pop_back();

            order_result.iter().for_each(|(x, t, v)| {
                self.asks.remove_order(*x);
                self.bids.remove_order(*t);

                match v {
                    None => {}
                    Some(a) => match a.action {
                        Side::BUY => self.bids.add_order(*a),
                        Side::SELL => self.asks.add_order(*a),
                    },
                }
            })
        }

        num_executions
    }

    fn cancel(&mut self, order: CancelOrder) -> bool {
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

    fn count_resting_orders(&mut self) -> usize {
        (self.asks.num_orders + self.bids.num_orders) as usize
    }
}
