use crate::api_spec::request::ApiOrderRequest;
use crate::app_state::AppState;
use axum::extract::State;
use axum::Json;
use common::types::order::{OrderRequest, TimeInForce};
use common::types::side::Side::{Buy, Sell};
use std::sync::Arc;

pub async fn client_order_to_api(
    State(state): State<Arc<AppState>>,
    Json(input): Json<ApiOrderRequest>,
) -> &'static str {
    let order_side = match input.side.to_lowercase().as_str() {
        "buy" | "bid" => Buy,
        "sell" | "ask" => Sell,
        _ => return "invalid side",
    };

    let msg = OrderRequest {
        client_id: input.client_id,
        instrument: instrument_str_to_char_buffer(&input.instrument),
        order_side,
        px: input.price,
        qty: input.qty,
        time_in_force: time_in_force_to_type(&input.time_in_force),
        timestamp: 0,
    };

    if let Err(_e) = state.tx_oe_api_queue.send(msg).await {
        println!("market-gateway channel closed");
        return "market-gateway down";
    }

    "order accepted"
}

fn instrument_str_to_char_buffer(symbol: &str) -> [u8; 16] {
    let bytes = symbol.as_bytes();
    let mut buf = [0u8; 16];
    let n = bytes.len().min(16);
    buf[..n].copy_from_slice(&bytes[..n]);
    buf
}

fn time_in_force_to_type(time_in_force: &str) -> TimeInForce {
    match time_in_force {
        "GTC" => TimeInForce::GTC,
        "IOC" => TimeInForce::IOC,
        "FOK" => TimeInForce::FOK,
        _ => panic!("Unknown time_in_force: {}", time_in_force),
    }
}
