mod market_data_book;
mod market_event;
mod process;

use crate::market_data_book::MarketDataBook;
use crate::market_event::MarketEvent;
use crate::process::engine_out_msg_receiver::initialize_engine_msg_out_receiver;
use axum::extract::ws::{Message, Utf8Bytes, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use common::transport::sequenced_message::SequencedEngineMessage;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::broadcast;

lazy_static! {
    pub static ref ENGINE_MSG_OUT_PORT: u16 = 3500;
}

#[tokio::main]
async fn main() {
    // Init channels
    let (tx_mdd_processor_to_ws, _) = broadcast::channel::<MarketEvent>(4096);
    let (tx_multicast_to_mdd_processor, mut rx_udp_to_mdd_processor) =
        tokio::sync::mpsc::unbounded_channel::<SequencedEngineMessage>();

    // Init MSG_OUT -> MDD mc recv thread
    let core_ids = core_affinity::get_core_ids()
        .unwrap()
        .into_iter()
        .collect::<Vec<_>>();
    let pinned_msg_out_core = core_ids[2];

    let engine_msg_out_thread = std::thread::spawn(move || {
        core_affinity::set_for_current(pinned_msg_out_core);
        initialize_engine_msg_out_receiver(*ENGINE_MSG_OUT_PORT, tx_multicast_to_mdd_processor)
            .unwrap();
    });

    // Init MDD processing thread
    let mdd_tx_mdd_processor_to_ws = tx_mdd_processor_to_ws.clone();
    tokio::spawn(async move {
        let mut book = MarketDataBook::new();

        let mut ticker = tokio::time::interval(Duration::from_millis(50));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                // Handle incoming engine messages
                maybe_msg = rx_udp_to_mdd_processor.recv() => {
                    match maybe_msg {
                        Some(msg) => {
                            book.update_from_engine(&msg.message);
                        }
                        None => break, // channel closed
                    }
                }

                // Emit book snapshot every 50ms regardless of activity
                _ = ticker.tick() => {
                    let event = book.generate_market_event();
                    let _ = mdd_tx_mdd_processor_to_ws.send(event);
                }
            }
        }
    });

    let app = Router::new()
        .route("/", get(root))
        .route("/ws/marketdata", get(ws_handler))
        .with_state(tx_mdd_processor_to_ws);

    println!("Market Data Distributor running on http://127.0.0.1:7000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:7000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    engine_msg_out_thread.join().unwrap();
}

async fn root() -> &'static str {
    "Market Data Distributor!"
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(broadcast_tx): State<broadcast::Sender<MarketEvent>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| ws_session(socket, broadcast_tx))
}

async fn ws_session(mut socket: WebSocket, broadcast_tx: broadcast::Sender<MarketEvent>) {
    println!("Client connected");
    let mut rx = broadcast_tx.subscribe();

    while let Ok(event) = rx.recv().await {
        let json = serde_json::to_string(&event).unwrap();
        if socket
            .send(Message::Text(Utf8Bytes::from(json)))
            .await
            .is_err()
        {
            break;
        }
    }

    println!("Client disconnected");
}
