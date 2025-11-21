use common::transport::sequenced_message::EngineMessage;
use tokio::sync::mpsc;

pub struct AppState {
    pub tx_oe_queue: mpsc::Sender<EngineMessage>,
}

impl AppState {
    pub fn new(tx_oe_queue: mpsc::Sender<EngineMessage>) -> AppState {
        AppState { tx_oe_queue }
    }
}
