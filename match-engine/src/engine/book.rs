use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::fmt::{Debug, Formatter};

use itertools::Itertools;

use common::message::{MarketDataEntry, MarketDataFullSnapshot, SnapshotType};

use crate::domain::order::Order;
use crate::domain::side::OrderAction;
use crate::domain::trade::Trade;

pub struct Book {
    asks: BinaryHeap<Order>,
    bids: BinaryHeap<Order>,
}

impl Book {
    pub fn new() -> Book {
        Book {
            bids: BinaryHeap::with_capacity(10000000),
            asks: BinaryHeap::with_capacity(10000000),
        }
    }

    pub fn apply_order(&mut self, order: Order) {
        match order.side {
            OrderAction::BUY => self.bids.push(order),
            OrderAction::SELL => self.asks.push(order),
        };
    }

    pub fn remove_order(&mut self, order: Order) {
        match order.side {
            OrderAction::BUY => self.bids.retain(|x| x.id != order.id),
            OrderAction::SELL => self.asks.retain(|x| x.id != order.id),
        };
    }

    pub fn create_book_snapshot(&mut self) -> MarketDataFullSnapshot {
        const MAX_SNAPSHOT_SIZE: usize = 10;

        let mut bids_snapshot: [MarketDataEntry; MAX_SNAPSHOT_SIZE] = [MarketDataEntry { px: 0, qty: 0 }; MAX_SNAPSHOT_SIZE];
        let mut asks_snapshot: [MarketDataEntry; MAX_SNAPSHOT_SIZE] = [MarketDataEntry { px: 0, qty: 0 }; MAX_SNAPSHOT_SIZE];

        let mut bids = self.bids.clone();
        let mut asks = self.asks.clone();

        for i in 0..MAX_SNAPSHOT_SIZE {
            if let Some(bid) = bids.pop() {
                bids_snapshot[i] = MarketDataEntry { px: bid.px, qty: bid.qty };
            }

            if let Some(ask) = asks.pop() {
                asks_snapshot[i] = MarketDataEntry { px: ask.px, qty: ask.qty };
            }
        }

        return MarketDataFullSnapshot {
            snapshot_type: SnapshotType::FullSnapshot,
            bids: bids_snapshot.iter()
                .take_while(|md_entry| md_entry.px > 0)
                .group_by(|order| order.px)
                .into_iter()
                .map(|(px, records)| {
                    let agg_qty = records.into_iter().fold(0, |mut aggregated_qty, nxt| {
                        aggregated_qty += nxt.px;
                        aggregated_qty
                    });

                    MarketDataEntry { px, qty: agg_qty }
                })
                .collect(),
            asks: asks_snapshot.iter()
                .rev()
                .take_while(|md_entry| md_entry.px > 0)
                .group_by(|order| order.px)
                .into_iter()
                .map(|(px, records)| {
                    let agg_qty = records.into_iter().fold(0, |mut aggregated_qty, nxt| {
                        aggregated_qty += nxt.px;
                        aggregated_qty
                    });

                    MarketDataEntry { px, qty: agg_qty }
                })
                .collect(),
        };
    }

    pub fn check_for_trades(&mut self) -> u32 {
        let mut executions: u32 = 0;
        while let (Some(ask), Some(bid)) = (self.asks.peek(), self.bids.peek()) {
            match self.attempt_order_match(ask, bid) {
                None => break,
                Some((_, remainder)) => {
                    executions += 1;
                    if let Some(rem) = remainder {
                        self.apply_order(rem);
                    }
                    // Matched orders ejected from book
                    self.asks.pop();
                    self.bids.pop();
                }
            }
        }

        return executions;
    }

    fn attempt_order_match(&self, ask: &Order, bid: &Order) -> Option<(Trade, Option<Order>)> {
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
                    Trade {
                        filled_quantity: quantity,
                        ask: ask.clone(),
                        bid: bid.clone(),
                    },
                    None,
                ))
            }
            Ordering::Greater => {
                let quantity = bid.qty;
                let mut remainder = ask.clone();
                remainder.qty -= quantity;
                Some((
                    Trade {
                        filled_quantity: quantity,
                        ask: ask.clone(),
                        bid: bid.clone(),
                    },
                    Some(remainder),
                ))
            }
            Ordering::Less => {
                let quantity = ask.qty;
                let mut remainder = bid.clone();
                remainder.qty -= quantity;
                Some((
                    Trade {
                        filled_quantity: quantity,
                        ask: ask.clone(),
                        bid: bid.clone(),
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

        let ask_vec: Vec<Order> = self
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
    use crate::domain::order::{Order, OrderType};
    use crate::domain::side::OrderAction;
    use crate::engine::book::Book;

    #[test]
    fn simple_like_for_like_match() {
        // Given
        let buy_order = Order::new(1, OrderType::New, 1, 10, OrderAction::BUY);
        let sell_order = Order::new(1, OrderType::New, 1, 10, OrderAction::SELL);

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
    fn buy_order_qty_remaining_on_book() {
        // Given
        let buy_order = Order::new(1, OrderType::New, 10, 1, OrderAction::BUY);
        let sell_order = Order::new(1, OrderType::New, 6, 1, OrderAction::SELL);

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
        let buy_order = Order::new(1, OrderType::New, 4, 1, OrderAction::BUY);
        let sell_order = Order::new(1, OrderType::New, 10, 1, OrderAction::SELL);

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
        let buy_order = Order::new(1, OrderType::New, 4, 1, OrderAction::BUY);
        let sell_order = Order::new(1, OrderType::New, 10, 1, OrderAction::SELL);

        let mut orderbook = Book::new();
        orderbook.apply_order(buy_order);
        orderbook.apply_order(sell_order);
        // When
        let cancel_order = Order::new(1, OrderType::Cancel, 0, 0, OrderAction::SELL);
        orderbook.remove_order(cancel_order);
        orderbook.check_for_trades();
        // Then
        assert_eq!(orderbook.bids.pop().unwrap().qty, 4);
        assert!(orderbook.asks.is_empty());
    }
}
