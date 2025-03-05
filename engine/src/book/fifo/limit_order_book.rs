use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::fmt::{Debug, Formatter};

use crate::book::book::Book;
use common::domain::execution::Execution;
use common::domain::order::{CancelOrder, LimitOrder};
use common::domain::domain::Side;
use common::util::time::epoch_nanos;
use rand::random;

pub struct LimitOrderBook {
    asks: BinaryHeap<LimitOrder>,
    bids: BinaryHeap<LimitOrder>,
}

impl LimitOrderBook {
    pub fn new() -> Self {
        Self {
            bids: BinaryHeap::with_capacity(50_000_000),
            asks: BinaryHeap::with_capacity(50_000_000),
        }
    }

    fn attempt_order_match(&self, ask: &LimitOrder, bid: &LimitOrder) -> Option<(Execution, Option<LimitOrder>)> {
        let (ask, bid) = match (ask.action, bid.action) {
            (Side::BUY, Side::SELL) => (bid, ask),
            (Side::SELL, Side::BUY) => (ask, bid),
            (_, _) => return None,
        };

        if ask.px > bid.px {
            return None;
        }

        match ask.qty.cmp(&bid.qty) {
            Ordering::Equal => {
                Some((
                    Execution {
                        id: random::<u32>(),
                        fill_qty: ask.qty,
                        ask: ask.clone(),
                        bid: bid.clone(),
                        execution_time: epoch_nanos(),
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
}

impl Book for LimitOrderBook {
    fn apply(&mut self, order: LimitOrder) {
        match order.action {
            Side::BUY => {
                self.bids.push(order);
            }
            Side::SELL => {
                self.asks.push(order);
            }
        };
    }

    fn check_for_trades(&mut self, max_execution_per_cycle: usize, arr: &mut [Execution]) -> usize {
        let mut num_executions: usize = 0;
        while let (Some(ask), Some(bid)) = (self.asks.peek(), self.bids.peek()) {
            if num_executions == max_execution_per_cycle - 1 {
                break;
            }

            match self.attempt_order_match(ask, bid) {
                None => break,
                Some((execution, remainder)) => {
                    // remove the match (any remainder is re-added
                    self.asks.pop();
                    self.bids.pop();

                    // move the execution to the outbound buffer
                    arr[num_executions] = execution;
                    num_executions += 1;

                    // add any remaining qty to the book
                    if let Some(rem) = remainder {
                        self.apply(rem);
                    }
                }
            }
        }
        num_executions
    }

    fn cancel(&mut self, order: CancelOrder) -> bool {
        match order.action {
            Side::BUY => self.bids.retain(|x| x.id != order.id),
            Side::SELL => self.asks.retain(|x| x.id != order.id)
        };
        true
    }

    fn count_resting_orders(&mut self) -> usize {
        self.asks.len() + self.bids.len()
    }
}

impl Debug for LimitOrderBook {
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
    use common::memory::memory::uninitialized_arr;

    use super::*;

    #[test]
    fn like_for_like_price_match() {
        // Given
        let buy_order = LimitOrder {
            client_id: 0,
            id: 1,
            action: Side::BUY,
            px: 1,
            qty: 10,
            placed_time: 0,
        };

        let sell_order = LimitOrder {
            client_id: 0,
            id: 1,
            action: Side::SELL,
            px: 1,
            qty: 10,
            placed_time: 0,
        };

        let mut orderbook = LimitOrderBook::new();
        orderbook.apply(buy_order);
        orderbook.apply(sell_order);
        // When
        let mut executions_buffer: [Execution; 10] = uninitialized_arr();

        orderbook.check_for_trades(10, &mut executions_buffer);        // Then
        assert!(orderbook.bids.is_empty());
        assert!(orderbook.asks.is_empty());
    }

    #[test]
    fn fifo_like_for_like_match() {
        // Given
        let buy_order = LimitOrder {
            client_id: 0,
            id: 1,
            action: Side::BUY,
            px: 1,
            qty: 10,
            placed_time: 0,
        };

        let sell_order = LimitOrder {
            client_id: 0,
            id: 2,
            action: Side::SELL,
            px: 1,
            qty: 10,
            placed_time: 0,
        };

        let latter_sell_order = LimitOrder {
            client_id: 0,
            id: 3,
            action: Side::SELL,
            px: 1,
            qty: 10,
            placed_time: 0,
        };

        let mut orderbook = LimitOrderBook::new();
        orderbook.apply(buy_order);
        orderbook.apply(sell_order);
        orderbook.apply(latter_sell_order);
        // When
        let mut executions_buffer: [Execution; 10] = uninitialized_arr();
        orderbook.check_for_trades(10, &mut executions_buffer);
        // Then
        assert!(orderbook.bids.is_empty());
        assert_eq!(*orderbook.asks.iter().next().unwrap(), latter_sell_order);
    }

    #[test]
    fn buy_order_qty_remaining_on_book() {
        // Given
        let buy_order = LimitOrder {
            client_id: 0,
            id: 1,
            action: Side::BUY,
            px: 1,
            qty: 10,
            placed_time: 0,
        };

        let sell_order = LimitOrder {
            client_id: 0,
            id: 1,
            action: Side::SELL,
            px: 1,
            qty: 6,
            placed_time: 0,
        };

        let mut orderbook = LimitOrderBook::new();
        orderbook.apply(buy_order);
        orderbook.apply(sell_order);
        // When
        let mut executions_buffer: [Execution; 10] = uninitialized_arr();
        orderbook.check_for_trades(10, &mut executions_buffer);
        // Then
        assert!(orderbook.asks.is_empty());
        assert_eq!(orderbook.bids.pop().unwrap().qty, 4)
    }

    #[test]
    fn sell_order_qty_remaining_on_book() {
        // Given
        let buy_order = LimitOrder {
            client_id: 0,
            id: 1,
            action: Side::BUY,
            px: 1,
            qty: 4,
            placed_time: 0,
        };

        let sell_order = LimitOrder {
            client_id: 0,
            id: 1,
            action: Side::SELL,
            px: 1,
            qty: 10,
            placed_time: 0,
        };

        let mut orderbook = LimitOrderBook::new();
        orderbook.apply(buy_order);
        orderbook.apply(sell_order);
        // When
        let mut executions_buffer: [Execution; 10] = uninitialized_arr();
        orderbook.check_for_trades(10, &mut executions_buffer);
        // Then
        assert!(orderbook.bids.is_empty());
        assert_eq!(orderbook.asks.pop().unwrap().qty, 6);
    }

    #[test]
    fn sell_order_cancel_removes_order_from_book() {
        // Given
        let buy_order = LimitOrder {
            client_id: 0,
            id: 1,
            action: Side::BUY,
            px: 1,
            qty: 4,
            placed_time: 0,
        };

        let sell_order = LimitOrder {
            client_id: 0,
            id: 1,
            action: Side::SELL,
            px: 1,
            qty: 10,
            placed_time: 0,
        };

        let mut orderbook = LimitOrderBook::new();
        orderbook.apply(buy_order);
        orderbook.apply(sell_order);
        // When
        let cancel_order = CancelOrder {
            client_id: 0,
            id: 1,
            action: Side::SELL,
        };
        orderbook.cancel(cancel_order);

        let mut executions_buffer: [Execution; 10] = uninitialized_arr();
        orderbook.check_for_trades(10, &mut executions_buffer);
        // Then
        assert_eq!(orderbook.bids.pop().unwrap().qty, 4);
        assert!(orderbook.asks.is_empty());
    }
}
