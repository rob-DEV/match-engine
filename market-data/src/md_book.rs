use common::message::side::Side;
use common::message::side::Side::BUY;
use common::transport::sequenced_message::EngineMessage;
use std::collections::{BTreeMap, HashMap};

#[derive(Debug)]
struct PriceLevel {
    px: u32,
    quantity: u32,
}

#[derive(Debug)]
pub struct MarketDataBook {
    bids: BTreeMap<u32, PriceLevel>, // descending order for best bid
    asks: BTreeMap<u32, PriceLevel>, // ascending order for best ask
    order_metadata: HashMap<u32, (Side, u32, u32)>,
}

impl MarketDataBook {
    pub fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            order_metadata: HashMap::new(),
        }
    }

    pub fn update_from_engine(&mut self, engine_message: &EngineMessage) {
        match engine_message {
            EngineMessage::NewOrderAck(new_order_ack) => {
                let book_side = if new_order_ack.side == BUY {
                    &mut self.bids
                } else {
                    &mut self.asks
                };
                let entry = book_side.entry(new_order_ack.px).or_insert(PriceLevel {
                    px: new_order_ack.px,
                    quantity: 0,
                });
                entry.quantity += new_order_ack.qty;

                self.order_metadata.insert(
                    new_order_ack.order_id,
                    (new_order_ack.side, new_order_ack.px, new_order_ack.qty),
                );
            }
            EngineMessage::CancelOrderAck(cancel_order_ack) => {
                let cancelled_order_metadata = self
                    .order_metadata
                    .get_mut(&cancel_order_ack.order_id)
                    .unwrap();

                let book_side = if cancelled_order_metadata.0 == BUY {
                    &mut self.bids
                } else {
                    &mut self.asks
                };

                let entry = book_side.get_mut(&cancelled_order_metadata.1).unwrap();

                entry.quantity -= cancelled_order_metadata.2;

                self.order_metadata.remove(&cancel_order_ack.order_id);
            }
            EngineMessage::TradeExecution(execution) => {
                let executed_qty = execution.exec_qty;

                if let Some(bid_order_metadata) =
                    self.order_metadata.get_mut(&execution.bid_order_id)
                {
                } else {
                }
                if let Some(ask_order_metadata) =
                    self.order_metadata.get_mut(&execution.ask_order_id)
                {
                } else {
                }
            }
            _ => {}
        }
    }

    pub fn emit_l1(&self) {
        let best_bid = self.bids.iter().rev().next();
        let best_ask = self.asks.iter().next();

        println!("--- L1 Top-of-Book ---");
        if let Some((_, bid)) = best_bid {
            println!("Best Bid: {} @ {}", bid.px, bid.quantity);
        }
        if let Some((_, ask)) = best_ask {
            println!("Best Ask: {} @ {}", ask.px, ask.quantity);
        }
    }

    pub fn emit_l2(&self) {
        println!("--- L2 Depth-of-Book ---");
        println!("Bids:");
        for (_, level) in self.bids.iter().rev() {
            println!("{} @ {}", level.px, level.quantity);
        }
        println!("Asks:");
        for (_, level) in self.asks.iter() {
            println!("{} @ {}", level.px, level.quantity);
        }
    }
}
