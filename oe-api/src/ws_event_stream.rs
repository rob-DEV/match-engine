use crate::api_spec::request::IncomingMessage;
use crate::api_spec::response::{
    ApiCancelOrderAckResponse, ApiExecutionReportResponse, ApiOrderAckResponse,
};
use crate::app_state::AppState;
use axum::extract::ws::{Message, Utf8Bytes};
use axum::extract::{Path, State, WebSocketUpgrade};
use axum::response::IntoResponse;
use common::transport::sequenced_message::EngineMessage;
use common::types::cancel_order::CancelOrderRequest;
use common::types::instrument::Instrument;
use common::types::order::{OrderRequest, TimeInForce};
use common::types::side::Side;
use common::util::time::system_nanos;
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::sync::mpsc;

pub async fn ws_event_stream(
    State(state): State<Arc<AppState>>,
    Path(client_id): Path<u32>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        println!("WS client {} connected", client_id);

        let last_heartbeat = Arc::new(AtomicU64::new(system_nanos()));
        // Channel for engine -> client messages
        let (tx, mut rx) = mpsc::channel::<EngineMessage>(128);
        state
            .tx_engine_to_client_channel
            .insert(client_id, tx.clone());

        let (mut ws_tx, mut ws_rx) = socket.split();

        let oe_api_to_client = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                let json = engine_msg_to_json(client_id, msg);
                println!("JSON from engine {}", json);
                if ws_tx
                    .send(Message::Text(Utf8Bytes::from(json)))
                    .await
                    .is_err()
                {
                    println!("WS send failed; closing");
                    break;
                }
            }
        });

        let state_oe_task = state.clone();
        let last_heartbeat_task = last_heartbeat.clone();
        let client_to_engine = tokio::spawn(async move {
            while let Some(msg_result) = ws_rx.next().await {
                match msg_result {
                    Ok(Message::Text(text)) => {
                        let engine_message = match serde_json::from_str::<IncomingMessage>(&text) {
                            Ok(msg) => match msg {
                                IncomingMessage::ApiOrderRequest(request) => {
                                    Some(EngineMessage::NewOrder(OrderRequest {
                                        client_id,
                                        instrument: Instrument::str_to_fixed_char_buffer(
                                            &request.instrument,
                                        ),
                                        order_side: Side::str_to_val(&request.side).unwrap(),
                                        px: request.px,
                                        qty: request.qty,
                                        time_in_force: TimeInForce::str_to_val(
                                            &request.time_in_force,
                                        )
                                        .unwrap(),
                                        timestamp: system_nanos(),
                                    }))
                                }
                                IncomingMessage::ApiOrderCancelRequest(request) => {
                                    Some(EngineMessage::CancelOrder(CancelOrderRequest {
                                        client_id,
                                        order_side: Side::str_to_val(&request.side).unwrap(),
                                        order_id: request.order_id,
                                        instrument: Instrument::str_to_fixed_char_buffer(
                                            &request.instrument,
                                        ),
                                    }))
                                }
                                IncomingMessage::Heartbeat(_) => {
                                    last_heartbeat_task.store(system_nanos(), Ordering::Relaxed);
                                    None
                                }
                            },
                            Err(e) => panic!("Error: {}", e),
                        };

                        // Send to engine required - heartbeats for example are not
                        if let Some(engine_message) = engine_message {
                            if state_oe_task
                                .tx_oe_api_queue
                                .send(engine_message)
                                .await
                                .is_err()
                            {
                                eprintln!("Send to engine failed; closing");
                            }
                        }
                    }

                    Ok(Message::Ping(_)) => {
                        println!("Received PING from client {}", client_id);
                    }
                    Ok(Message::Pong(_)) => {
                        println!("Received PONG from client {}", client_id);
                    }

                    Ok(Message::Close(_)) => break,

                    Ok(_) => {} // other

                    Err(e) => {
                        println!("WS error: {e}");
                        break;
                    }
                }
            }
        });

        let last_heartbeat_guard_task = last_heartbeat.clone();
        let all_task_heartbeat_guard = tokio::spawn(async move {
            use tokio::time::{Duration, sleep};

            loop {
                sleep(Duration::from_secs(5)).await;

                let now = system_nanos();

                if now - last_heartbeat_guard_task.load(Ordering::Acquire)
                    > Duration::from_secs(10).as_nanos() as u64
                {
                    println!("Client {} heartbeat timeout!", client_id);

                    oe_api_to_client.abort();
                    client_to_engine.abort();
                    break;
                }
            }
        });

        let _ = tokio::join!(all_task_heartbeat_guard);

        // Cleanup
        state.tx_engine_to_client_channel.remove(&client_id);
        println!("WS client {} disconnected", client_id);
    })
}
fn engine_msg_to_json(client_id: u32, msg: EngineMessage) -> String {
    match msg {
        EngineMessage::NewOrderAck(a) => serde_json::to_string(&ApiOrderAckResponse {
            client_id: a.client_id,
            instrument: "BTC-USD".into(),
            order_id: a.order_id,
            side: Side::val_to_str(a.side),
            px: a.px,
            qty: a.qty,
            ack_time: a.ack_time,
        })
        .unwrap(),
        EngineMessage::CancelOrderAck(a) => serde_json::to_string(&ApiCancelOrderAckResponse {
            client_id: a.client_id,
            instrument: "BTC-USD".into(),
            order_id: a.order_id,
            cancel_order_status: "".into(),
            reason: "".into(),
            ack_time: a.ack_time,
        })
        .unwrap(),
        EngineMessage::TradeExecution(e) => serde_json::to_string(&ApiExecutionReportResponse {
            client_id,
            instrument: "BTC-USD".into(),
            order_id: e.bid_order_id,
            fill_type: "".into(),
            exec_px: e.exec_px,
            exec_qty: e.exec_qty,
            exec_type: "".into(),
            exec_ns: e.exec_ns,
        })
        .unwrap(),
        _ => panic!("Unexpected message {:?}", msg),
    }
}
