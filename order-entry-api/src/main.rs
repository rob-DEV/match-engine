use std::env;
use std::net::SocketAddr;

use axum::Json;
use axum::response::IntoResponse;
use axum::routing::post;
use lazy_static::lazy_static;
use serde_json::json;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use common::message::GatewayMessage;

lazy_static! {
    pub static ref ENGINE_PORT: u16 = env::var("ENGINE_PORT").unwrap_or("3000".to_owned()).parse::<u16>().unwrap();
    pub static ref API_PORT: u16 = env::var("API_PORT").unwrap_or("3001".to_owned()).parse::<u16>().unwrap();
}

#[tokio::main]
async fn main() {
    let app = axum::Router::new()
        .route("/order", post(connection_handler))
        .route("/md", post(connection_handler));

    println!("Starting Order Entry API on port {}", *API_PORT);
    let listener = tokio::net::TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], *API_PORT)))
        .await
        .unwrap();

    axum::serve(listener, app)
        .await
        .unwrap();
}

async fn connection_handler(Json(payload): Json<GatewayMessage>) -> impl IntoResponse {
    let mut engine_socket = TcpStream::connect(SocketAddr::from(([0, 0, 0, 0], *ENGINE_PORT))).await.unwrap();

    match serde_json::to_vec(&payload) {
        Ok(bytes) => engine_socket.write_all(&*bytes).await.unwrap(),
        Err(err) => panic!("Error {}", err)
    };

    let mut ack_buffer: [u8; 4096] = [0; 4096];

    match engine_socket.read(&mut ack_buffer).await {
        Ok(bytes) => {
            let order_ack_message: GatewayMessage = serde_json::from_slice(&ack_buffer[..bytes]).unwrap();
            return Json(serde_json::to_value(&order_ack_message).unwrap());
        }
        Err(_) => {}
    }

    Json(json!(""))
}