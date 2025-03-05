use crate::engine;
use common::config::config::load_engine_config;
use common::domain::messaging::SequencedEngineMessage;
use common::domain::order::Order;
use std::sync::mpsc::{Receiver, Sender};

pub fn initialize_match_thread(engine_msg_out_tx: Sender<SequencedEngineMessage>, order_entry_rx: Receiver<Order>) {
    let config = load_engine_config();
    let mut match_engine = engine::match_engine::MatchEngine::new(config.get("symbol").unwrap().to_owned(), config.get("isin").unwrap().to_owned());
    match_engine.run(order_entry_rx, engine_msg_out_tx);
}