use crate::market_event::{L2Level, MarketEvent, Trade, L1, L2};
use common::transport::sequenced_message::EngineMessage;
use common::types::execution_report::{ExecType, ExecutionReport};
use common::types::new_order::NewOrderAck;
use common::types::side::Side;
use common::types::side::Side::Buy;
use common::types::side::Side::Sell;
use std::collections::{BTreeMap, HashMap};

#[derive(Debug)]
struct PriceLevel {
    px: u32,
    qty: u32,
}

#[derive(Debug)]
struct OrderMetadata {
    pub side: Side,
    pub px: u32,
    pub qty: u32,
}

impl OrderMetadata {
    pub fn new(side: Side, px: u32, qty: u32) -> Self {
        OrderMetadata { side, px, qty }
    }
}

struct TradeMetadata {
    pub exec_px: u32,
    pub exec_qty: u32,
    pub exec_ns: u64,
}

impl TradeMetadata {
    pub fn new(execution_report: &ExecutionReport) -> Self {
        TradeMetadata {
            exec_px: execution_report.exec_px,
            exec_qty: execution_report.exec_qty,
            exec_ns: execution_report.exec_ns,
        }
    }
}

const MAX_MARKET_EVENT_DEPTH: usize = 10;

pub struct MarketDataBook {
    bids_levels: BTreeMap<u32, PriceLevel>,
    asks_levels: BTreeMap<u32, PriceLevel>,
    order_metadata_map: HashMap<u32, OrderMetadata>,
    orders: u32,

    // stats
    last_trade_px: u32,
    last_trades: [Trade; MAX_MARKET_EVENT_DEPTH],
    trade_count: usize,
}

impl MarketDataBook {
    pub fn new() -> Self {
        Self {
            bids_levels: BTreeMap::new(),
            asks_levels: BTreeMap::new(),
            order_metadata_map: HashMap::new(),
            orders: 0,
            last_trade_px: 0,
            last_trades: [Trade::default(); MAX_MARKET_EVENT_DEPTH],
            trade_count: 0,
        }
    }

    pub fn update_from_engine(&mut self, engine_message: &EngineMessage) -> bool {
        match engine_message {
            EngineMessage::NewOrderAck(new_order_ack) => self.update_new(new_order_ack),
            EngineMessage::CancelOrderAck(cancel_order_ack) => {
                self.update_cancel(cancel_order_ack.order_id)
            }
            EngineMessage::TradeExecution(execution) => match execution.exec_type {
                ExecType::MatchEvent => {
                    self.update_execution(&execution, TradeMetadata::new(execution))
                }
                ExecType::SelfMatchPrevented => self.update_smp_execution(&execution),
            },
            _ => {}
        }

        return true;
    }

    pub fn generate_market_event(&self) -> MarketEvent {
        let best_bid = self.bids_levels.keys().max().unwrap_or(&0);
        let best_ask = self.asks_levels.keys().min().unwrap_or(&0);

        let mut l2_snapshot = L2 {
            bids: [L2Level::default(); 10],
            asks: [L2Level::default(); 10],
        };

        for (idx, level) in self
            .bids_levels
            .iter()
            .rev()
            .take(MAX_MARKET_EVENT_DEPTH)
            .enumerate()
        {
            l2_snapshot.bids[idx].px = level.1.px;
            l2_snapshot.bids[idx].qty = level.1.qty;
        }
        for (idx, level) in self
            .asks_levels
            .iter()
            .take(MAX_MARKET_EVENT_DEPTH)
            .enumerate()
        {
            l2_snapshot.asks[idx].px = level.1.px;
            l2_snapshot.asks[idx].qty = level.1.qty;
        }

        MarketEvent {
            l1: L1 {
                best_bid: *best_bid,
                best_ask: *best_ask,
                last_price: self.last_trade_px,
            },
            l2: l2_snapshot,
            last_px: self.last_trade_px,
            trades: self.last_trades,
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

    pub fn emit_recent_trades(&self) {
        println!("Trades:");
        for i in 0..self.trade_count {
            let exec = &self.last_trades[i];
            println!("px:{} @ qty:{} @ {} ", exec.px, exec.qty, exec.ts);
        }
    }

    pub fn order_count(&self) -> u32 {
        self.orders
    }

    fn update_new(&mut self, new_order_ack: &NewOrderAck) {
        let book_side = if new_order_ack.side == Buy {
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
            OrderMetadata::new(new_order_ack.side, new_order_ack.px, new_order_ack.qty),
        );

        self.orders += 1;
    }

    fn update_execution(&mut self, execution: &ExecutionReport, metadata: TradeMetadata) {
        let executed_qty = execution.exec_qty;

        if let Some(bid_order_metadata) = self.order_metadata_map.get_mut(&execution.bid_order_id) {
            let bid_px = bid_order_metadata.px;
            // cleanup order
            bid_order_metadata.qty -= executed_qty;

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

        self.last_trade_px = metadata.exec_px;
        self.last_trades[self.trade_count % MAX_MARKET_EVENT_DEPTH] = Trade {
            px: metadata.exec_px,
            qty: metadata.exec_qty,
            ts: metadata.exec_ns,
        };
        self.trade_count += 1;
    }

    fn update_smp_execution(&mut self, execution: &ExecutionReport) {
        let smp_order_id = if execution.bid_order_id != 0 {
            execution.bid_order_id
        } else {
            execution.ask_order_id
        };

        self.update_cancel(smp_order_id)
    }

    fn update_cancel(&mut self, cancel_order_id: u32) {
        if let Some(order_metadata) = self.order_metadata_map.get(&cancel_order_id) {
            let order_side = order_metadata.side;
            let order_px = order_metadata.px;
            let order_qty = order_metadata.qty;

            let side_price_level_treemap = match order_side {
                Buy => &mut self.bids_levels,
                Sell => &mut self.asks_levels,
            };

            let price_level = side_price_level_treemap.get_mut(&order_px).unwrap();

            price_level.qty -= order_qty;
            if price_level.qty == 0 {
                side_price_level_treemap.remove(&order_px);
            }

            self.order_metadata_map.remove(&cancel_order_id);
        }
    }
}
