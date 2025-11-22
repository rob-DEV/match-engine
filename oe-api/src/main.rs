mod api_spec;
mod app_state;
mod engine_event_stream;
mod engine_order_entry;
mod ws_event_stream;
use crate::app_state::AppState;
use crate::engine_event_stream::gateway_event_stream;
use crate::engine_order_entry::gateway_order_entry;
use crate::ws_event_stream::ws_event_stream;
use axum::routing::get;
use axum::Router;
use common::transport::sequenced_message::EngineMessage;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (tx_oe_api_queue, rx_oe_api_queue) = mpsc::channel::<EngineMessage>(10_000);

    let (rx_gw_stream, tx_gateway_stream) = TcpStream::connect("127.0.0.1:3001")
        .await
        .expect("cannot connect to TCP market-gateway")
        .into_split();

    let state = Arc::new(AppState::new(tx_oe_api_queue));

    tokio::spawn(gateway_order_entry(tx_gateway_stream, rx_oe_api_queue));
    tokio::spawn(gateway_event_stream(rx_gw_stream, state.clone()));

    println!("Connected to market-gateway");

    let app = Router::new()
        .route("/ws/event_stream/{client_id}", get(ws_event_stream))
        .with_state(state);

    println!("Order API listening on 0.0.0.0:8080");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
