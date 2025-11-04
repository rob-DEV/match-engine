use crate::engine;
use common::config::config::load_engine_config;
use common::message::instrument::Instrument;
use common::transport::sequenced_message::SequencedEngineMessage;
use crate::domain::order::Order;
use std::sync::mpsc::{Receiver, Sender};

pub fn initialize_match_thread(
    engine_msg_out_tx: Sender<SequencedEngineMessage>,
    order_entry_rx: Receiver<Order>,
) {
    let config = load_engine_config();
    let mut match_engine = engine::match_engine::MatchEngine::new(Instrument {
        id: config.get("id").unwrap().parse::<u32>().unwrap(),
        symbol: config.get("symbol").unwrap().to_string(),
        isin: config.get("isin").unwrap().to_string(),
    });
    match_engine.run(order_entry_rx, engine_msg_out_tx);
}
