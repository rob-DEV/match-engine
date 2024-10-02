use std::cmp::Ordering;
use std::collections::{BinaryHeap};
use std::fmt::{Debug, Formatter};

use rand::random;

use common::engine::OrderAction;

use crate::domain::execution::{Execution, FullMatch, PartialMatch};
use crate::domain::order::{CancelOrder, LimitOrder};
use crate::util::time::epoch_nanos;

pub struct CentralLimitOrderBook {
    asks: BinaryHeap<LimitOrder>,
    bids: BinaryHeap<LimitOrder>,
}

impl CentralLimitOrderBook {
    pub fn new() -> CentralLimitOrderBook {
        CentralLimitOrderBook {
            bids: BinaryHeap::with_capacity(500_000),
            asks: BinaryHeap::with_capacity(500_000),
        }
    }

    pub fn apply_order(&mut self, order: LimitOrder) {
        match order.action {
            OrderAction::BUY => {
                self.bids.push(order);
            }
            OrderAction::SELL => {
                self.asks.push(order);
            }
        };
    }

    pub fn remove_order(&mut self, order: CancelOrder) {
        match order.action {
            OrderAction::BUY => {
                let mut px = 0;
                let mut qty = 0;
                for it in self.bids.iter() {
                    if it.id == order.id {
                        px = it.px;
                        qty = it.qty;
                        break;
                    }
                }

                self.bids.retain(|x| x.id != order.id);
            }
            OrderAction::SELL => {
                let mut px = 0;
                let mut qty = 0;
                for it in self.asks.iter() {
                    if it.id == order.id {
                        px = it.px;
                        qty = it.qty;
                        break;
                    }
                }
                self.asks.retain(|x| x.id != order.id);
            }
        };
    }

    pub fn check_for_trades(&mut self, max_execution_per_cycle: usize, arr: &mut [Execution]) -> usize {
        let mut executions: usize = 0;
        while let (Some(ask), Some(bid)) = (self.asks.peek(), self.bids.peek()) {
            if executions == max_execution_per_cycle - 1 {
                break;
            }

            match self.attempt_order_match(ask, bid) {
                None => break,
                Some((execution, remainder)) => {
                    // remove the match (any remainder is re-added
                    self.asks.pop();
                    self.bids.pop();

                    // move the execution to the outbound buffer
                    executions += 1;
                    arr[executions] = execution;

                    // add any remaining qty to the book
                    if let Some(rem) = remainder {
                        self.apply_order(rem);
                    }
                }
            }
        }

        return executions;
    }

    fn attempt_order_match(&self, ask: &LimitOrder, bid: &LimitOrder) -> Option<(Execution, Option<LimitOrder>)> {
        let (ask, bid) = match (ask.action, bid.action) {
            (OrderAction::BUY, OrderAction::SELL) => (bid, ask),
            (OrderAction::SELL, OrderAction::BUY) => (ask, bid),
            (_, _) => return None,
        };

        if ask.px > bid.px {
            return None;
        }

        match ask.qty.cmp(&bid.qty) {
            Ordering::Equal => {
                Some((
                    Execution::FullMatch(FullMatch {
                        id: random::<u32>(),
                        ask: ask.clone(),
                        bid: bid.clone(),
                        execution_time: epoch_nanos(),
                    }),
                    None,
                ))
            }
            Ordering::Greater => {
                let quantity = bid.qty;
                let mut remainder = ask.clone();
                remainder.qty -= quantity;

                Some((
                    Execution::PartialMatch(PartialMatch {
                        id: random::<u32>(),
                        fill_qty: quantity,
                        ask: ask.clone(),
                        bid: bid.clone(),
                        execution_time: epoch_nanos(),
                    }),
                    Some(remainder),
                ))
            }
            Ordering::Less => {
                let quantity = ask.qty;
                let mut remainder = bid.clone();
                remainder.qty -= quantity;
                Some((
                    Execution::PartialMatch(PartialMatch {
                        id: random::<u32>(),
                        fill_qty: quantity,
                        ask: ask.clone(),
                        bid: bid.clone(),
                        execution_time: epoch_nanos(),
                    }),
                    Some(remainder),
                ))
            }
        }
    }
}

impl Debug for CentralLimitOrderBook {
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
    use crate::util::memory::uninitialized_arr;

    use super::*;

    #[test]
    fn like_for_like_price_match() {
        // Given
        let buy_order = LimitOrder {
            id: 1,
            action: OrderAction::BUY,
            px: 1,
            qty: 10,
            placed_time: 0,
        };

        let sell_order = LimitOrder {
            id: 1,
            action: OrderAction::SELL,
            px: 1,
            qty: 10,
            placed_time: 0,
        };

        let mut orderbook = CentralLimitOrderBook::new();
        orderbook.apply_order(buy_order);
        orderbook.apply_order(sell_order);
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
            id: 1,
            action: OrderAction::BUY,
            px: 1,
            qty: 10,
            placed_time: 0,
        };

        let sell_order = LimitOrder {
            id: 2,
            action: OrderAction::SELL,
            px: 1,
            qty: 10,
            placed_time: 0,
        };

        let latter_sell_order = LimitOrder {
            id: 3,
            action: OrderAction::SELL,
            px: 1,
            qty: 10,
            placed_time: 0,
        };

        let mut orderbook = CentralLimitOrderBook::new();
        orderbook.apply_order(buy_order);
        orderbook.apply_order(sell_order);
        orderbook.apply_order(latter_sell_order);
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
            id: 1,
            action: OrderAction::BUY,
            px: 1,
            qty: 10,
            placed_time: 0,
        };

        let sell_order = LimitOrder {
            id: 1,
            action: OrderAction::SELL,
            px: 1,
            qty: 6,
            placed_time: 0,
        };

        let mut orderbook = CentralLimitOrderBook::new();
        orderbook.apply_order(buy_order);
        orderbook.apply_order(sell_order);
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
            id: 1,
            action: OrderAction::BUY,
            px: 1,
            qty: 4,
            placed_time: 0,
        };

        let sell_order = LimitOrder {
            id: 1,
            action: OrderAction::SELL,
            px: 1,
            qty: 10,
            placed_time: 0,
        };

        let mut orderbook = CentralLimitOrderBook::new();
        orderbook.apply_order(buy_order);
        orderbook.apply_order(sell_order);
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
            id: 1,
            action: OrderAction::BUY,
            px: 1,
            qty: 4,
            placed_time: 0,
        };

        let sell_order = LimitOrder {
            id: 1,
            action: OrderAction::SELL,
            px: 1,
            qty: 10,
            placed_time: 0,
        };

        let mut orderbook = CentralLimitOrderBook::new();
        orderbook.apply_order(buy_order);
        orderbook.apply_order(sell_order);
        // When
        let cancel_order = CancelOrder {
            id: 1,
            action: OrderAction::SELL,
        };
        orderbook.remove_order(cancel_order);

        let mut executions_buffer: [Execution; 10] = uninitialized_arr();
        orderbook.check_for_trades(10, &mut executions_buffer);
        // Then
        assert_eq!(orderbook.bids.pop().unwrap().qty, 4);
        assert!(orderbook.asks.is_empty());
    }
}
