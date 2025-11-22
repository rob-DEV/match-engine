use common::transport::sequenced_message::EngineMessage;
use dashmap::DashMap;
use tokio::sync::mpsc;

pub struct AppState {
    pub tx_oe_api_queue: mpsc::Sender<EngineMessage>,
    pub tx_engine_to_client_channel: DashMap<u32, mpsc::Sender<EngineMessage>>,
    pub last_client_heartbeat: DashMap<u32, u64>,
}

impl AppState {
    pub fn new(tx_oe_to_gateway: mpsc::Sender<EngineMessage>) -> AppState {
        AppState {
            tx_oe_api_queue: tx_oe_to_gateway,
            tx_engine_to_client_channel: DashMap::new(),
            last_client_heartbeat: DashMap::new(),
        }
    }
}
