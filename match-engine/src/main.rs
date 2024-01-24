use std::sync::mpsc::{channel, Receiver, Sender};

use crate::engine::domain::Order;

mod engine;

#[tokio::main]
async fn main() {
    let (order_entry_tx, engine_order_rx): (Sender<Order>, Receiver<Order>) = channel();

    // Engine started on separate non-tokio thread
    let match_engine = engine::match_engine::MatchEngine::new();
    match_engine.run(engine_order_rx);

    // Order entry tokio rt
    let match_server = engine::match_server::MatchServer::new();
    match_server.await.run(order_entry_tx).await;
}