// FIFO book

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::fmt::{Debug, Formatter};
use crate::engine::types::{Order, Side, Trade};

pub struct Orderbook {
    asks: BinaryHeap<Order>,
    bids: BinaryHeap<Order>,
}

impl Orderbook {
    pub fn new() -> Orderbook {
        Orderbook {
            asks: Default::default(),
            bids: Default::default(),
        }
    }

    pub fn apply_order(&mut self, order: Order) {
        match order.side {
            Side::BUY => self.bids.push(order),
            Side::SELL => self.asks.push(order),
        };
    }

    pub fn check_for_trades(&mut self) -> Vec<Trade> {
        let mut executed_trades: Vec<Trade> = Vec::new();

        while let (Some(ask), Some(bid)) = (self.asks.peek(), self.bids.peek()) {
            match self.attempt_order_match(ask, bid) {
                None => break,
                Some((trade, remainder)) => {
                    // Match and trade executed: pop bid and ask
                    self.asks.pop();
                    self.bids.pop();
                    executed_trades.push(trade);
                    if let Some(rem) = remainder {
                        match rem.side {
                            Side::BUY => self.bids.push(rem),
                            Side::SELL => self.asks.push(rem),
                        };
                    }
                }
            }
        }
        return executed_trades;
    }

    fn attempt_order_match(&self, ask: &Order, bid: &Order) -> Option<(Trade, Option<Order>)> {
        let (ask, bid) = match (ask.side, bid.side) {
            (Side::BUY, Side::SELL) => (bid, ask),
            (Side::SELL, Side::BUY) => (ask, bid),
            (_, _) => return None
        };

        if ask.price > bid.price {
            return None;
        }

        match ask.quantity.cmp(&bid.quantity) {
            Ordering::Equal => {
                let quantity = ask.quantity;
                Some((Trade { filled_quantity: quantity, ask: ask.clone(), bid: bid.clone() }, None))
            }
            Ordering::Greater => {
                let quantity = bid.quantity;
                let mut remainder = ask.clone();
                remainder.quantity -= quantity;
                Some((Trade { filled_quantity: quantity, ask: ask.clone(), bid: bid.clone() }, Some(remainder)))
            }
            Ordering::Less => {
                let quantity = ask.quantity;
                let mut remainder = bid.clone();
                remainder.quantity -= quantity;
                Some((Trade { filled_quantity: quantity, ask: ask.clone(), bid: bid.clone() }, Some(remainder)))
            }
        }
    }
}

impl Debug for Orderbook {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "-------------------Orderbook-------------------").unwrap();
        writeln!(f, "{0: <10} | {1: <10} | {2: <10} | {3: <10}", "ClientId", "B/S", "Qty", "Px").unwrap();

        for bid in &self.bids.clone().into_sorted_vec() {
            writeln!(f, "{0: <10} | {1: <10} | {2: <10} | {3: <10}", bid.client_id, "BUY", bid.quantity, bid.price).unwrap();
        }
        writeln!(f, "-----------------------------------------------").unwrap();
        for ask in &self.asks.clone().into_sorted_vec() {
            writeln!(f, "{0: <10} | {1: <10} | {2: <10} | {3: <10}", ask.client_id, "SELL", ask.quantity, ask.price).unwrap();
        }
        writeln!(f, "{0: <10} | {1: <10} | {2: <10} | {3: <10}", "ClientId", "B/S", "Qty", "Px").unwrap();
        write!(f, "-----------------Orderbook End-----------------")
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::types::Side;
    use crate::{Order, Orderbook};

    #[test]
    fn simple_like_for_like_match() {
        // Given
        let buy_order = Order::new(1, 1, 1, 10, Side::BUY);
        let sell_order = Order::new(1, 1, 1, 10, Side::SELL);

        let mut order_book = Orderbook::new();
        order_book.apply_order(buy_order);
        order_book.apply_order(sell_order);
        // When
        order_book.check_for_trades();
        // Then
        assert_eq!(order_book.bids.is_empty(), true);
        assert_eq!(order_book.asks.is_empty(), true);
    }

    #[test]
    fn buy_order_qty_remaining_on_book() {
        // Given
        let buy_order = Order::new(1, 1, 1, 10, Side::BUY);
        let sell_order = Order::new(1, 1, 1, 6, Side::SELL);

        let mut order_book = Orderbook::new();
        order_book.apply_order(buy_order);
        order_book.apply_order(sell_order);
        // When
        order_book.check_for_trades();
        // Then
        assert_eq!(order_book.asks.is_empty(), true);
        assert_eq!(order_book.bids.pop().unwrap().quantity, 4)
    }

    #[test]
    fn sell_order_qty_remaining_on_book() {
        // Given
        let buy_order = Order::new(1, 1, 1, 4, Side::BUY);
        let sell_order = Order::new(1, 1, 1, 10, Side::SELL);

        let mut order_book = Orderbook::new();
        order_book.apply_order(buy_order);
        order_book.apply_order(sell_order);
        // When
        order_book.check_for_trades();
        // Then
        assert_eq!(order_book.bids.is_empty(), true);
        assert_eq!(order_book.asks.pop().unwrap().quantity, 6);
    }
}