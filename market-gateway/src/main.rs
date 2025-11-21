mod app_state;
mod message;
mod process;

use crate::app_state::AppState;

use crate::process::engine_msg_in_thread::msg_in_thread;
use crate::process::engine_msg_out_thread::msg_out_thread;
use crate::process::gateway_to_oe_api::gateway_to_oe_api_handler;
use crate::process::oe_api_to_gateway::oe_api_to_gateway_handler;
use common::transport::sequenced_message::EngineMessage;
use core_affinity::CoreId;
use std::error::Error;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{broadcast, mpsc};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("--- Initializing Gateway ---");

    let listener = TcpListener::bind("0.0.0.0:3001").await?;
    println!("Gateway listening on 3001");

    let (tx_gw_queue, rx_gw_queue) = mpsc::channel::<EngineMessage>(1024);
    let (tx_engine_queue, _) = broadcast::channel::<EngineMessage>(1024);

    let state = Arc::new(AppState::new(tx_gw_queue));

    msg_in_thread(3000, CoreId { id: 0 }, rx_gw_queue);
    msg_out_thread(3500, CoreId { id: 0 }, tx_engine_queue.clone());

    loop {
        let (socket, addr) = listener.accept().await?;
        let state = state.clone();

        let (rx_oe_api, tx_oe_api) = socket.into_split();

        tokio::spawn(async move {
            if let Err(e) = oe_api_to_gateway_handler(rx_oe_api, state).await {
                eprintln!("Connection {} error: {}", addr, e);
            }
        });

        let tx_engine_queue_task = tx_engine_queue.clone();
        tokio::spawn(async move {
            if let Err(e) =
                gateway_to_oe_api_handler(tx_oe_api, tx_engine_queue_task.subscribe()).await
            {
                eprintln!("Connection {} error: {}", addr, e);
            }
        });
    }
}
