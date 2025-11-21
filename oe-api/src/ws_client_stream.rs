use crate::app_state::AppState;
use axum::extract::{State, WebSocketUpgrade};
use axum::response::IntoResponse;
use std::sync::Arc;
use tokio::sync::broadcast;

pub async fn ws_engine_to_client_feed(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    client_id: axum::extract::Path<u64>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        
    })
}
