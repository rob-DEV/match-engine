use std::env;
use std::sync::{Arc, mpsc, Mutex};

use common::message::MarketDataFullSnapshot;

use crate::domain::order::Order;

mod engine;
mod domain;
mod memory;

#[tokio::main]
async fn main() {
    // Engine Channels - Order Entry & Market Data
    let (order_entry_tx, order_entry_rx): (mpsc::Sender<Order>, mpsc::Receiver<Order>) = mpsc::channel();
    let market_data_snapshot_mutex = Arc::new(Mutex::new(MarketDataFullSnapshot::new()));

    // Engine started on separate non-tokio thread
    let md_mutex = market_data_snapshot_mutex.clone();
    let match_engine = engine::match_engine::MatchEngine::new(order_entry_rx, md_mutex);
    match_engine.run();

    // Order entry tokio rt
    let app_port = env::var("APP_PORT").unwrap_or("3000".to_string());

    let md_mutex = market_data_snapshot_mutex.clone();
    let match_server = engine::match_server::MatchServer::new(app_port, order_entry_tx, md_mutex);
    match_server.await.run().await;
}