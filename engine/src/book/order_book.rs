use crate::book::book::Book;
use crate::book::book_side::BookSide;
use common::types::cancel_order::CancelOrderRequest;
use crate::domain::order::LimitOrder;
use common::types::side::Side::{BUY, SELL};

pub type Price = u32;
pub struct LimitOrderBook {
    pub asks: BookSide,
    pub bids: BookSide,
}

impl LimitOrderBook {
    pub fn new() -> Self {
        Self {
            asks: BookSide::new(SELL),
            bids: BookSide::new(BUY),
        }
    }
}

impl Book for LimitOrderBook {
    fn add_order(&mut self, order: LimitOrder) {
        match order.side {
            BUY => {
                self.bids.add_order(order);
            }
            SELL => {
                self.asks.add_order(order);
            }
        };
    }

    fn remove_order(&mut self, order: &CancelOrderRequest) -> bool {
        match order.order_side {
            BUY => {
                self.bids.remove_order(order.order_id);
            }
            SELL => {
                self.asks.remove_order(order.order_id);
            }
        };

        true
    }

    fn orders_on_book(&mut self) -> usize {
        (self.asks.num_orders() + self.bids.num_orders()) as usize
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
