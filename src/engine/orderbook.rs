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
            // Match the optional of the merge
            match self.merge_orders(ask, bid) {
                None => break,
                Some((trade, remainder)) => {
                    self.asks.pop();
                    self.bids.pop();
                    executed_trades.push(trade);
                    if let Some(rem) = remainder {
                        // Apply the remainder to the order book
                        // iff the order has only been partially filled
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

    // Merge and return trade, also return any remaining orders that need filled
    fn merge_orders(&self, ask: &Order, bid: &Order) -> Option<(Trade, Option<Order>)> {
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