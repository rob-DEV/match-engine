use std::env;
use std::net::SocketAddr;

use axum::Json;
use axum::response::{IntoResponse, Response};
use axum::routing::post;

use crate::domain::domain::AuthenticationResponse;

mod domain;

#[tokio::main]
async fn main() {
    let app_host = env::var("APP_HOST").unwrap_or("0.0.0.0".to_string());
    let app_port = env::var("APP_PORT").unwrap_or("3000".to_string());

    let app = axum::Router::new()
        .route("/auth", post(auth))
        .route("/order", post(order_entry))
        .route("/order_status", post(order_status));

    let listener = tokio::net::TcpListener::bind(SocketAddr::new(app_host.parse().unwrap(), app_port.parse().unwrap())).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn auth(Json(data): Json<serde_json::Value>) -> Response {
    let authentication_response = AuthenticationResponse {
        token: data.get("token").map(|t| t.as_str().unwrap()).unwrap_or("NOT_AUTHENTICATED").to_owned()
    };

    Json(authentication_response).into_response()
}

async fn order_entry(Json(data): Json<serde_json::Value>) -> String {
    format!("DATA: {:?}", data)
}

async fn order_status(Json(data): Json<serde_json::Value>) -> String {
    format!("DATA: {:?}", data)
}