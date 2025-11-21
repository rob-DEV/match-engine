use common::types::execution_report::ExecutionReport;
use common::types::order::OrderRequest;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};

#[derive(Clone)]
pub struct AppState {
    pub tx_oe_api_queue: mpsc::Sender<OrderRequest>,
    pub connected_client_state: Arc<RwLock<HashMap<u64, broadcast::Sender<ExecutionReport>>>>,
}

impl AppState {
    pub fn new(tx_oe_to_gateway: mpsc::Sender<OrderRequest>) -> AppState {
        AppState {
            tx_oe_api_queue: tx_oe_to_gateway,
            connected_client_state: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}
