use std::env;
use std::net::SocketAddr;

use axum::Json;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use common::message::GatewayMessage;

#[tokio::main]
async fn main() {
    let app_host = env::var("APP_HOST").unwrap_or("127.0.0.1".to_string());
    let app_port = env::var("APP_PORT").unwrap_or("3001".to_string());

    let app = axum::Router::new()
        .route("/", get(health_check_handler))
        .route("/order", post(order_entry_handler))
        .route("/md", post(market_data_handler));


    let listener = tokio::net::TcpListener::bind(SocketAddr::new(app_host.parse().unwrap(), app_port.parse().unwrap()))
        .await
        .unwrap();

    println!("Order Entry API server started on port {}", app_port);

    axum::serve(listener, app)
        .await
        .unwrap();
}

async fn health_check_handler() -> impl IntoResponse {
    let json_response = serde_json::json!({
        "status": "ok",
        "message": "Order Entry API"
    });

    Json(json_response)
}

async fn order_entry_handler(Json(payload): Json<GatewayMessage>) -> impl IntoResponse {
    let mut stream = TcpStream::connect("127.0.0.1:3000").await.unwrap();
    return match payload {
        GatewayMessage::NewOrder(_) => {
            match serde_json::to_vec(&payload) {
                Ok(bytes) => stream.write_all(&*bytes).await.unwrap(),
                Err(err) => panic!("Error {}", err)
            };

            let mut ack_buffer: [u8; 512] = [0; 512];
            let result = stream.read(&mut ack_buffer).await.unwrap();

            if result > 0 {
                let order_ack_message: GatewayMessage = serde_json::from_slice(&ack_buffer[..result]).unwrap();
                return Json(serde_json::to_value(&order_ack_message).unwrap());
            }

            Json(serde_json::json!({"status": "ok","message": "Order Entry API"}))
        }
        _ => Json(serde_json::json!({"status": "ok","message": "Order Entry API"}))
    }
}

async fn market_data_handler(Json(payload): Json<GatewayMessage>) -> impl IntoResponse {
    let mut stream = TcpStream::connect("127.0.0.1:3000").await.unwrap();
    return match payload {
        GatewayMessage::MarketDataRequest(_) => {
            match serde_json::to_vec(&payload) {
                Ok(bytes) => stream.write_all(&*bytes).await.unwrap(),
                Err(err) => panic!("Error {}", err)
            };

            let mut ack_buffer: [u8; 512] = [0; 512];
            let result = stream.read(&mut ack_buffer).await.unwrap();

            if result > 0 {
                let order_ack_message: GatewayMessage = serde_json::from_slice(&ack_buffer[..result]).unwrap();
                return Json(serde_json::to_value(&order_ack_message).unwrap());
            }

            Json(serde_json::json!({"status": "ok","message": "Order Entry API"}))
        }
        _ => Json(serde_json::json!({"status": "ok","message": "Order Entry API"}))
    }
}