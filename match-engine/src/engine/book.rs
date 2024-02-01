use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use common::domain::{CancelOrder, MarketDataEntry, MarketDataFullSnapshot};
use common::domain::OrderAction;

use crate::domain::execution::Execution;
use crate::domain::order::LimitOrder;

pub struct Book {
    asks: BinaryHeap<LimitOrder>,
    bids: BinaryHeap<LimitOrder>,
}

impl Book {
    pub fn new() -> Book {
        Book {
            bids: BinaryHeap::with_capacity(500_000),
            asks: BinaryHeap::with_capacity(500_000),
        }
    }

    pub fn apply_order(&mut self, order: LimitOrder) {
        match order.side {
            OrderAction::BUY => self.bids.push(order),
            OrderAction::SELL => self.asks.push(order),
        };
    }

    pub fn remove_order(&mut self, order: CancelOrder) {
        match order.action {
            OrderAction::BUY => self.bids.retain(|x| x.id != order.id),
            OrderAction::SELL => self.asks.retain(|x| x.id != order.id),
        };
    }

    pub fn populate_md_mutex(&mut self, md_mutex: &Arc<Mutex<MarketDataFullSnapshot>>) {
        const MAX_SNAPSHOT_SIZE: usize = 20;

        let mut asks_md = self.asks.clone();
        let mut bids_md = self.bids.clone();

        let mut md_asks: usize = 0;
        let mut md_bids: usize = 0;

        let mut snapshot = md_mutex.lock().unwrap();

        while md_asks < MAX_SNAPSHOT_SIZE || md_bids < MAX_SNAPSHOT_SIZE {
            let ask = asks_md.pop();
            let bid = bids_md.pop();

            if let Some(ask) = ask {
                snapshot.asks.push(MarketDataEntry {
                    px: ask.px,
                    qty: ask.qty,
                });

                md_asks += 1;
            } else {
                break;
            }

            if let Some(bid) = bid {
                snapshot.bids.push(MarketDataEntry {
                    px: bid.px,
                    qty: bid.qty,
                });

                md_bids += 1;
            } else {
                break;
            }
        }
    }

    pub fn check_for_trades(&mut self, max_execution_per_cycle: usize, arr: &mut [Execution]) -> usize {
        let mut executions: usize = 0;
        while let (Some(ask), Some(bid)) = (self.asks.peek(), self.bids.peek()) {
            if executions == max_execution_per_cycle {
                break;
            }

            match self.attempt_order_match(ask, bid) {
                None => break,
                Some((execution, remainder)) => {
                    if let Some(rem) = remainder {
                        self.apply_order(rem);
                    }

                    arr[executions] = execution;

                    executions += 1;

                    self.asks.pop();
                    self.bids.pop();
                }
            }
        }

        return executions;
    }

    fn attempt_order_match(&self, ask: &LimitOrder, bid: &LimitOrder) -> Option<(Execution, Option<LimitOrder>)> {
        let (ask, bid) = match (ask.side, bid.side) {
            (OrderAction::BUY, OrderAction::SELL) => (bid, ask),
            (OrderAction::SELL, OrderAction::BUY) => (ask, bid),
            (_, _) => return None,
        };

        if ask.px > bid.px {
            return None;
        }

        match ask.qty.cmp(&bid.qty) {
            Ordering::Equal => {
                let quantity = ask.qty;
                Some((
                    Execution {
                        fill_qty: quantity,
                        ask: ask.clone(),
                        bid: bid.clone(),
                        execution_time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos(),
                    },
                    None,
                ))
            }
            Ordering::Greater => {
                let quantity = bid.qty;
                let mut remainder = ask.clone();
                remainder.qty -= quantity;
                Some((
                    Execution {
                        fill_qty: quantity,
                        ask: ask.clone(),
                        bid: bid.clone(),
                        execution_time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos(),
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
                        fill_qty: quantity,
                        ask: ask.clone(),
                        bid: bid.clone(),
                        execution_time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos(),
                    },
                    Some(remainder),
                ))
            }
        }
    }
}

impl Debug for Book {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "-------------------Order Book-------------------").unwrap();
        writeln!(
            f,
            "{0: <10} | {1: <10} | {2: <10} | {3: <10}",
            "ClientId", "B/S", "Qty", "Px"
        ).unwrap();

        for bid in &self.bids.clone().into_sorted_vec() {
            writeln!(
                f,
                "{0: <10} | {1: <10} | {2: <10} | {3: <10}",
                bid.id, "BUY", bid.qty, bid.px
            ).unwrap();
        }
        writeln!(f, "-----------------------------------------------").unwrap();

        let ask_vec: Vec<LimitOrder> = self
            .asks
            .clone()
            .into_sorted_vec()
            .into_iter()
            .rev()
            .collect();

        for ask in &ask_vec {
            writeln!(
                f,
                "{0: <10} | {1: <10} | {2: <10} | {3: <10}",
                ask.id, "SELL", ask.qty, ask.px
            )
                .unwrap();
        }
        writeln!(
            f,
            "{0: <10} | {1: <10} | {2: <10} | {3: <10}",
            "ClientId", "B/S", "Qty", "Px"
        )
            .unwrap();
        write!(f, "-----------------Order Book End-----------------")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn like_for_like_price_match() {
        // Given
        let buy_order = LimitOrder {
            id: 1,
            px: 1,
            qty: 10,
            side: OrderAction::BUY,
            placed_time: 0,
        };

        let sell_order = LimitOrder {
            id: 1,
            px: 1,
            qty: 10,
            side: OrderAction::SELL,
            placed_time: 0,
        };

        let mut orderbook = Book::new();
        orderbook.apply_order(buy_order);
        orderbook.apply_order(sell_order);
        // When
        orderbook.check_for_trades();
        // Then
        assert!(orderbook.bids.is_empty());
        assert!(orderbook.asks.is_empty());
    }

    #[test]
    fn fifo_like_for_like_match() {
        // Given
        let buy_order = LimitOrder {
            id: 1,
            px: 1,
            qty: 10,
            side: OrderAction::BUY,
            placed_time: 0,
        };

        let sell_order = LimitOrder {
            id: 2,
            px: 1,
            qty: 10,
            side: OrderAction::SELL,
            placed_time: 0,
        };

        let latter_sell_order = LimitOrder {
            id: 3,
            px: 1,
            qty: 10,
            side: OrderAction::SELL,
            placed_time: 0,
        };

        let mut orderbook = Book::new();
        orderbook.apply_order(buy_order);
        orderbook.apply_order(sell_order);
        orderbook.apply_order(latter_sell_order);
        // When
        orderbook.check_for_trades();
        // Then
        assert!(orderbook.bids.is_empty());
        assert_eq!(*orderbook.asks.iter().next().unwrap(), latter_sell_order);
    }

    #[test]
    fn buy_order_qty_remaining_on_book() {
        // Given
        let buy_order = LimitOrder {
            id: 1,
            px: 1,
            qty: 10,
            side: OrderAction::BUY,
            placed_time: 0,
        };

        let sell_order = LimitOrder {
            id: 1,
            px: 1,
            qty: 6,
            side: OrderAction::SELL,
            placed_time: 0,
        };

        let mut orderbook = Book::new();
        orderbook.apply_order(buy_order);
        orderbook.apply_order(sell_order);
        // When
        orderbook.check_for_trades();
        // Then
        assert!(orderbook.asks.is_empty());
        assert_eq!(orderbook.bids.pop().unwrap().qty, 4)
    }

    #[test]
    fn sell_order_qty_remaining_on_book() {
        // Given
        let buy_order = LimitOrder {
            id: 1,
            px: 1,
            qty: 4,
            side: OrderAction::BUY,
            placed_time: 0,
        };

        let sell_order = LimitOrder {
            id: 1,
            px: 1,
            qty: 10,
            side: OrderAction::SELL,
            placed_time: 0,
        };

        let mut orderbook = Book::new();
        orderbook.apply_order(buy_order);
        orderbook.apply_order(sell_order);
        // When
        orderbook.check_for_trades();
        // Then
        assert!(orderbook.bids.is_empty());
        assert_eq!(orderbook.asks.pop().unwrap().qty, 6);
    }

    #[test]
    fn sell_order_cancel_removes_order_from_book() {
        // Given
        let buy_order = LimitOrder {
            id: 1,
            px: 1,
            qty: 4,
            side: OrderAction::BUY,
            placed_time: 0,
        };

        let sell_order = LimitOrder {
            id: 1,
            px: 1,
            qty: 10,
            side: OrderAction::SELL,
            placed_time: 0,
        };

        let mut orderbook = Book::new();
        orderbook.apply_order(buy_order);
        orderbook.apply_order(sell_order);
        // When
        let cancel_order = CancelOrder {
            action: OrderAction::SELL,
            id: 1,
        };
        orderbook.remove_order(cancel_order);
        orderbook.check_for_trades();
        // Then
        assert_eq!(orderbook.bids.pop().unwrap().qty, 4);
        assert!(orderbook.asks.is_empty());
    }
}
