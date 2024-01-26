use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::fmt::{Debug, Formatter};

use crate::domain::order::Order;
use crate::domain::side::Side;
use crate::domain::trade::Trade;

pub struct OrderBook {
    asks: BinaryHeap<Order>,
    bids: BinaryHeap<Order>,
}

impl OrderBook {
    pub fn new() -> OrderBook {
        OrderBook {
            asks: BinaryHeap::with_capacity(10000000),
            bids: BinaryHeap::with_capacity(10000000),
        }
    }

    pub fn apply_order(&mut self, order: Order) {
        match order.side {
            Side::BUY => self.bids.push(order),
            Side::SELL => self.asks.push(order),
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
            (Side::BUY, Side::SELL) => (bid, ask),
            (Side::SELL, Side::BUY) => (ask, bid),
            (_, _) => return None,
        };

        if ask.price > bid.price {
            return None;
        }

        match ask.quantity.cmp(&bid.quantity) {
            Ordering::Equal => {
                let quantity = ask.quantity;
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
                let quantity = bid.quantity;
                let mut remainder = ask.clone();
                remainder.quantity -= quantity;
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
                let quantity = ask.quantity;
                let mut remainder = bid.clone();
                remainder.quantity -= quantity;
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

impl Debug for OrderBook {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "-------------------Orderbook-------------------").unwrap();
        writeln!(
            f,
            "{0: <10} | {1: <10} | {2: <10} | {3: <10}",
            "ClientId", "B/S", "Qty", "Px"
        ).unwrap();

        for bid in &self.bids.clone().into_sorted_vec() {
            writeln!(
                f,
                "{0: <10} | {1: <10} | {2: <10} | {3: <10}",
                bid.identifier, "BUY", bid.quantity, bid.price
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
                ask.identifier, "SELL", ask.quantity, ask.price
            )
                .unwrap();
        }
        writeln!(
            f,
            "{0: <10} | {1: <10} | {2: <10} | {3: <10}",
            "ClientId", "B/S", "Qty", "Px"
        )
            .unwrap();
        write!(f, "-----------------Orderbook End-----------------")
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::order::Order;
    use crate::domain::side::Side;
    use crate::engine::order_book::OrderBook;

    #[test]
    fn simple_like_for_like_match() {
        // Given
        let buy_order = Order::new(1, 1, 10, Side::BUY);
        let sell_order = Order::new(1, 1, 10, Side::SELL);

        let mut orderbook = OrderBook::new();
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
        let buy_order = Order::new(1, 10, 1, Side::BUY);
        let sell_order = Order::new(1, 6, 1, Side::SELL);

        let mut orderbook = OrderBook::new();
        orderbook.apply_order(buy_order);
        orderbook.apply_order(sell_order);
        // When
        orderbook.check_for_trades();
        // Then
        assert!(orderbook.asks.is_empty());
        assert_eq!(orderbook.bids.pop().unwrap().quantity, 4)
    }

    #[test]
    fn sell_order_qty_remaining_on_book() {
        // Given
        let buy_order = Order::new(1, 4, 1, Side::BUY);
        let sell_order = Order::new(1, 10, 1, Side::SELL);

        let mut orderbook = OrderBook::new();
        orderbook.apply_order(buy_order);
        orderbook.apply_order(sell_order);
        // When
        orderbook.check_for_trades();
        // Then
        assert!(orderbook.bids.is_empty());
        assert_eq!(orderbook.asks.pop().unwrap().quantity, 6);
    }
}
