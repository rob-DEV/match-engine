use common::memory::memory::uninitialized_arr;
use common::transport::sequenced_message::EngineMessage;
use common::types::execution_report::{ExecType, ExecutionReport};
use common::types::new_order::NewOrderAck;
use common::types::side::Side;
use common::types::side::Side::BUY;
use common::types::side::Side::SELL;
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

const MAX_RECENT_TRADE_SIZE: usize = 20;

pub struct MarketDataBook {
    bids_levels: BTreeMap<u32, PriceLevel>,
    asks_levels: BTreeMap<u32, PriceLevel>,
    order_metadata_map: HashMap<u32, OrderMetadata>,
    orders: u32,
    recent_trades: [TradeMetadata; MAX_RECENT_TRADE_SIZE],
    recent_trades_count: usize,
}

impl MarketDataBook {
    pub fn new() -> Self {
        Self {
            bids_levels: BTreeMap::new(),
            asks_levels: BTreeMap::new(),
            order_metadata_map: HashMap::new(),
            orders: 0,
            recent_trades: uninitialized_arr(),
            recent_trades_count: 0,
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
        for i in 0..self.recent_trades_count {
            let exec = &self.recent_trades[i];
            println!(
                "px:{} @ qty:{} @ {} ",
                exec.exec_px, exec.exec_qty, exec.exec_ns
            );
        }
    }

    pub fn order_count(&self) -> u32 {
        self.orders
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

        self.recent_trades[self.recent_trades_count] = metadata;
        self.recent_trades_count += 1;
        if self.recent_trades_count == MAX_RECENT_TRADE_SIZE {
            self.recent_trades_count = 0;
        }
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
                BUY => &mut self.bids_levels,
                SELL => &mut self.asks_levels,
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
