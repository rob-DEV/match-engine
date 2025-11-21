use common::transport::sequenced_message::EngineMessage;
use dashmap::DashMap;
use tokio::sync::mpsc;

pub struct AppState {
    pub tx_oe_api_queue: mpsc::Sender<EngineMessage>,
    pub tx_client_id_channel_map: DashMap<u32, mpsc::Sender<EngineMessage>>,
}

impl AppState {
    pub fn new(tx_oe_to_gateway: mpsc::Sender<EngineMessage>) -> AppState {
        AppState {
            tx_oe_api_queue: tx_oe_to_gateway,
            tx_client_id_channel_map: DashMap::new(),
        }
    }
}
