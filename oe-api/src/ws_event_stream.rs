use crate::api_spec::request::ApiOrderRequest;
use crate::api_spec::response::{
    ApiCancelOrderAckResponse, ApiExecutionAckResponse, ApiOrderAckResponse,
};
use crate::app_state::AppState;
use axum::extract::ws::Message;
use axum::extract::{Path, State, WebSocketUpgrade};
use axum::response::IntoResponse;
use common::transport::sequenced_message::EngineMessage;
use common::types::instrument::Instrument;
use common::types::order::{OrderRequest, TimeInForce};
use common::types::side::Side;
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::mpsc;

pub async fn ws_engine_event_stream(
    State(state): State<Arc<AppState>>,
    Path(client_id): Path<u32>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        println!("WS client {} connected", client_id);
        let (tx, mut rx) = mpsc::channel::<EngineMessage>(10_000);
        state.tx_client_id_channel_map.insert(client_id, tx);

        let (mut ws_sender, mut ws_receiver) = socket.split();

        // TASK 1: outbound engine-event → websocket
        let outbound = async {
            while let Some(msg) = rx.recv().await {
                let json = engine_msg_to_json(msg);

                if ws_sender.send(Message::Text(json.into())).await.is_err() {
                    break;
                }
            }
        };

        // TASK 2: inbound websocket messages (pings, closes)
        let inbound = async {
            while let Some(Ok(msg)) = ws_receiver.next().await {
                if let Message::Text(json) = msg {
                    if let Ok(order) = serde_json::from_str::<ApiOrderRequest>(&json) {
                        let msg = OrderRequest {
                            client_id: order.client_id,
                            instrument: Instrument::instrument_str_to_fixed_buffer(
                                &order.instrument,
                            ),
                            order_side: Side::str_to_type(&order.side),
                            px: order.price,
                            qty: order.qty,
                            time_in_force: TimeInForce::str_to_type(&order.time_in_force),
                            timestamp: 0,
                        };

                        let engine_msg = EngineMessage::NewOrder(msg);

                        if state.tx_oe_api_queue.send(engine_msg).await.is_err() {
                            println!("engine channel closed");
                            break;
                        }
                    }
                }
            }
        };

        // Run both until one stops
        tokio::select! {
            _ = outbound => {},
            _ = inbound => {},
        }

        // CLEANUP
        state.tx_client_id_channel_map.remove(&client_id);
    })
}

fn engine_msg_to_json(msg: EngineMessage) -> String {
    match msg {
        EngineMessage::NewOrderAck(a) => serde_json::to_string(&ApiOrderAckResponse {
            client_id: a.client_id,
            instrument: "BTC-USD".into(),
            side: "".into(),
            px: a.px,
            qty: a.qty,
            ack_time: a.ack_time,
        })
        .unwrap(),
        EngineMessage::CancelOrderAck(a) => serde_json::to_string(&ApiCancelOrderAckResponse {
            client_id: a.client_id,
            order_id: a.order_id,
            cancel_order_status: "".into(),
            reason: "".into(),
            ack_time: a.ack_time,
        })
        .unwrap(),
        EngineMessage::TradeExecution(e) => serde_json::to_string(&ApiExecutionAckResponse {
            client_id: e.bid_client_id,
            order_id: e.bid_order_id,
            fill_type: "".into(),
            exec_px: e.exec_px,
            exec_qty: e.exec_qty,
            exec_type: "".into(),
            exec_ns: e.exec_ns,
        })
        .unwrap(),
        _ => panic!("Unexpected message"),
    }
}
