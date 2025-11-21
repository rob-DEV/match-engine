mod api_spec;
mod api_to_engine_handler;
mod app_state;
mod engine_connection;
mod ws_client_stream;

use crate::api_to_engine_handler::client_order_to_api;
use crate::app_state::AppState;
use crate::engine_connection::market_gateway_sender;
use crate::ws_client_stream::ws_engine_to_client_feed;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{routing::post, Router};
use common::types::order::OrderRequest;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::{io::AsyncWriteExt, sync::mpsc};

#[tokio::main]
async fn main() {
    let (tx_oe_api_queue, rx_oe_api_queue) = mpsc::channel::<OrderRequest>(10_000);

    tokio::spawn(market_gateway_sender(rx_oe_api_queue));

    let state = Arc::new(AppState::new(tx_oe_api_queue));

    let app = Router::new()
        .route("/order", post(client_order_to_api))
        .route("/ws/trade_feed", get(ws_engine_to_client_feed))
        .with_state(state);

    println!("Order API listening on 0.0.0.0:8080");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:7000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
