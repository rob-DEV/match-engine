use crate::domain::order::Order;
use crate::engine::match_engine::MatchEngine;
use common::transport::sequenced_message::EngineMessage;
use core_affinity::CoreId;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;

pub fn match_thread(
    engine_msg_out_tx: Sender<EngineMessage>,
    order_entry_rx: Receiver<Order>,
    pinned_match_core: CoreId,
) -> JoinHandle<()> {
    thread::spawn(move || {
        core_affinity::set_for_current(pinned_match_core);
        let mut match_engine = MatchEngine::new();
        match_engine.run(order_entry_rx, engine_msg_out_tx);
    })
}
