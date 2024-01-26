use std::env;
use std::net::SocketAddr;

use axum::Json;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use rand::Rng;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

mod domain;

#[tokio::main]
async fn main() {
    let app_host = env::var("APP_HOST").unwrap_or("127.0.0.1".to_string());
    let app_port = env::var("APP_PORT").unwrap_or("3000".to_string());

    let app = axum::Router::new()
        .route("/", get(health_check_handler))
        .route("/order", post(order_entry));

    println!("✅ Server started successfully");

    let listener = tokio::net::TcpListener::bind(SocketAddr::new(app_host.parse().unwrap(), app_port.parse().unwrap()))
        .await
        .unwrap();
    axum::serve(listener, app)
        .await
        .unwrap();
}

async fn health_check_handler() -> impl IntoResponse {
    const MESSAGE: &str = "API Services";

    let json_response = serde_json::json!({
        "status": "ok",
        "message": MESSAGE
    });

    Json(json_response)
}

async fn order_entry(Json(_data): Json<serde_json::Value>) {
    let mut stream = TcpStream::connect("127.0.0.1:8080").await.unwrap();

    if rand::thread_rng().gen_range(0..10) % 2 == 0 {
        stream.write_all(format!("B|{}|{}", rand::thread_rng().gen_range(0..100), rand::thread_rng().gen_range(0..100)).as_bytes()).await.unwrap();
    } else {
        stream.write_all(format!("S|{}|{}", rand::thread_rng().gen_range(0..100), rand::thread_rng().gen_range(0..100)).as_bytes()).await.unwrap();
    }
}