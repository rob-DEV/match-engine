use std::env;
use std::net::SocketAddr;

use axum::Json;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use serde_json::json;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use common::message::GatewayMessage;

#[tokio::main]
async fn main() {
    let app_port = env::var("APP_PORT").unwrap_or("3001".to_string());

    let app = axum::Router::new()
        .route("/", get(health_handler))
        .route("/order", post(connection_handler))
        .route("/md", post(connection_handler));

    println!("Starting Order Entry API on port {}", app_port);

    let listener = tokio::net::TcpListener::bind(SocketAddr::new("127.0.0.1".parse().unwrap(), app_port.parse().unwrap()))
        .await
        .unwrap();

    axum::serve(listener, app)
        .await
        .unwrap();
}

async fn health_handler() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "message": "Order Entry API"
    }))
}

async fn connection_handler(Json(payload): Json<GatewayMessage>) -> impl IntoResponse {
    let mut engine_socket = TcpStream::connect("127.0.0.1:3000").await.unwrap();

    match serde_json::to_vec(&payload) {
        Ok(bytes) => engine_socket.write_all(&*bytes).await.unwrap(),
        Err(err) => panic!("Error {}", err)
    };

    let mut ack_buffer: [u8; 512] = [0; 512];

    match engine_socket.read(&mut ack_buffer).await {
        Ok(bytes) => {
            let order_ack_message: GatewayMessage = serde_json::from_slice(&ack_buffer[..bytes]).unwrap();
            return Json(serde_json::to_value(&order_ack_message).unwrap());
        }
        Err(_) => {}
    }

    Json(json!(""))
}