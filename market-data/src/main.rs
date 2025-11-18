mod process;
mod market_data_book;
mod market_event;

use axum::{
    routing::{get, post},
    http::StatusCode,
    Json, Router,
};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use crate::process::engine_out_msg_receiver::initialize_engine_msg_out_receiver;

lazy_static! {
    pub static ref ENGINE_MSG_OUT_PORT: u16 = 3500;
}

#[tokio::main]
async fn main() {

    // Init MSG_OUT -> MDD mc recv thread
    initialize_engine_msg_out_receiver(*ENGINE_MSG_OUT_PORT)
        .expect("failed to initialize engine msg_out -> mdd");

    // Init MDD processing thread


    let app = Router::new()
        .route("/", get(root));

    println!("Market Data Distributor running on http://127.0.0.1:7000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:7000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Market Data Distributor!"
}