use common::message::cancel_order::CancelOrderAck;
use common::message::execution::TradeExecution;
use common::message::new_order::NewOrderAck;
use common::message::side::Side::BUY;
use common::transport::sequenced_message::EngineMessage;
use std::collections::{BTreeMap, HashMap};

#[derive(Debug)]
struct PriceLevel {
    px: u32,
    qty: u32,
}

struct OrderMetadata {
    pub px: u32,
    pub qty: u32,
}

impl OrderMetadata {
    pub fn new(px: u32, qty: u32) -> Self {
        OrderMetadata { px, qty }
    }
}

pub struct MarketDataBook {
    bids_levels: BTreeMap<u32, PriceLevel>,
    asks_levels: BTreeMap<u32, PriceLevel>,
    order_metadata_map: HashMap<u32, OrderMetadata>,
}

impl MarketDataBook {
    pub fn new() -> Self {
        Self {
            bids_levels: BTreeMap::new(),
            asks_levels: BTreeMap::new(),
            order_metadata_map: HashMap::new(),
        }
    }

    pub fn update_from_engine(&mut self, engine_message: &EngineMessage) {
        match engine_message {
            EngineMessage::NewOrderAck(new_order_ack) => self.update_new(new_order_ack),
            EngineMessage::CancelOrderAck(cancel_order_ack) => {
                self.update_cancel(&cancel_order_ack)
            }
            EngineMessage::TradeExecution(execution) => {
                self.update_execution(&execution);
            }
            _ => {}
        }
    }

    pub fn emit_l1(&self) {
        let best_bid = self.bids_levels.iter().rev().next();
        let best_ask = self.asks_levels.iter().next();

        println!("--- L1 Top-of-Book ---");
        if let Some((_, bid)) = best_bid {
            println!("Best Bid: {} @ {}", bid.px, bid.qty);
        }
        if let Some((_, ask)) = best_ask {
            println!("Best Ask: {} @ {}", ask.px, ask.qty);
        }
    }

    pub fn emit_l2(&self) {
        let max_depth = 10;

        println!("--- L2 Depth-of-Book ---");
        println!("Bids:");
        for (_, level) in self.bids_levels.iter().rev().take(max_depth) {
            println!("{} @ {}", level.px, level.qty);
        }
        println!("Asks:");
        for (_, level) in self.asks_levels.iter().take(max_depth) {
            println!("{} @ {}", level.px, level.qty);
        }
    }

    fn update_new(&mut self, new_order_ack: &NewOrderAck) {
        let book_side = if new_order_ack.side == BUY {
            &mut self.bids_levels
        } else {
            &mut self.asks_levels
        };
        let entry = book_side.entry(new_order_ack.px).or_insert(PriceLevel {
            px: new_order_ack.px,
            qty: 0,
        });
        entry.qty += new_order_ack.qty;

        self.order_metadata_map.insert(
            new_order_ack.order_id,
            OrderMetadata::new(new_order_ack.px, new_order_ack.qty),
        );
    }

    fn update_execution(&mut self, execution: &TradeExecution) {
        let executed_qty = execution.exec_qty;

        if let Some(bid_order_metadata) = self.order_metadata_map.get_mut(&execution.bid_order_id) {
            let bid_px = bid_order_metadata.px;
            // cleanup order
            bid_order_metadata.qty -= executed_qty;
            assert!(bid_order_metadata.qty >= 0);

            if bid_order_metadata.qty == 0 {
                self.order_metadata_map.remove(&execution.bid_order_id);
            }

            // cleanup level
            let price_level = self.bids_levels.get_mut(&bid_px).unwrap();
            price_level.qty -= executed_qty;
            if price_level.qty == 0 {
                self.bids_levels.remove(&bid_px);
            }
        }

        if let Some(ask_order_metadata) = self.order_metadata_map.get_mut(&execution.ask_order_id) {
            let ask_px = ask_order_metadata.px;
            // cleanup order
            ask_order_metadata.qty -= executed_qty;
            assert!(ask_order_metadata.qty >= 0);

            if ask_order_metadata.qty == 0 {
                self.order_metadata_map.remove(&execution.ask_order_id);
            }

            // cleanup level
            let price_level = self.asks_levels.get_mut(&ask_px).unwrap();
            price_level.qty -= executed_qty;
            if price_level.qty == 0 {
                self.asks_levels.remove(&ask_px);
            }
        }
    }
    fn update_cancel(&mut self, cancel_order_ack: &CancelOrderAck) {}
}
