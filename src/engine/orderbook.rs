// FIFO book

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use crate::engine::types::{Order, Side, Trade};

#[derive(Debug)]
pub struct Orderbook {
    asks: BinaryHeap<Order>,
    bids: BinaryHeap<Order>
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

        while !self.asks.is_empty() && !self.bids.is_empty() {
            let (ask, bid) = (self.asks.pop().unwrap(), self.bids.pop().unwrap());
            // Match the optional of the merge
            match self.merge_orders(ask, bid) {
                None => break,
                Some((trade, remainder)) => {
                    executed_trades.push(trade);
                    if let Some(rem) = remainder {
                        // Apply the remainder to the order book
                        // iff the order has only been partially filled
                        match rem.side {
                            Side::BUY => self.bids.push(rem.clone()),
                            Side::SELL => self.asks.push(rem.clone()),
                        };
                    }
                }
            }
        }
        return executed_trades;
    }

    pub fn check_for_trades_stack_based_results(&mut self) -> Vec<Trade> {
        let mut executed_trades: Vec<Trade> = Vec::new();

        while !self.asks.is_empty() && !self.bids.is_empty() {
            let (ask, bid) = (self.asks.pop().unwrap(), self.bids.pop().unwrap());
            // Match the optional of the merge
            match self.merge_orders(ask, bid) {
                None => break,
                Some((trade, remainder)) => {
                    executed_trades.push(trade);
                    if let Some(rem) = remainder {
                        // Apply the remainder to the order book
                        // iff the order has only been partially filled

                        match rem.side {
                            Side::BUY => self.bids.push(rem.clone()),
                            Side::SELL => self.asks.push(rem.clone()),
                        };
                    }
                }
            }
        }

        return executed_trades;
    }

    // Merge and return trade, also return any remaining orders that need filled
    fn merge_orders(&self, ask: Order, bid: Order) -> Option<(Trade, Option<Order>)> {
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
                Some((Trade { filled_quantity: quantity, ask, bid }, None))
            }
            Ordering::Greater => {
                let quantity = bid.quantity;
                let mut remainder = ask.clone();
                remainder.quantity -= quantity;
                Some((Trade { filled_quantity: quantity, ask, bid }, Some(remainder)))
            }
            Ordering::Less => {
                let quantity = ask.quantity;
                let mut remainder = bid.clone();
                remainder.quantity -= quantity;
                Some((Trade { filled_quantity: quantity, ask, bid }, Some(remainder)))
            }
        }

    }


}