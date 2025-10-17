use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use common::domain::messaging::EngineMessage;
use std::error::Error;
use std::net::SocketAddr;
use crate::client_state::on_client_connection;
use crate::GATEWAY_PORT;
use crate::message::GatewayMessage;
use crate::parser::MessageConverter;

pub async fn initialize_gateway_session_handler(
    inbound_engine_message_tx: Sender<GatewayMessage>,
    session_msg_tx_map: Arc<Mutex<HashMap<u32, Sender<EngineMessage>>>>,
) -> Result<(), Box<dyn Error>> {
    // Shared for now
    let message_converter = Arc::new(Mutex::new(MessageConverter::new()));
    let tcp_listener =
        tokio::net::TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], *GATEWAY_PORT))).await?;

    println!("Initialized Gateway FIX session handler {}", *GATEWAY_PORT);

    loop {
        let connection = tcp_listener.accept().await?;
        let task_message_converter = message_converter.clone();
        let task_inbound_engine_message_tx = inbound_engine_message_tx.clone();
        let task_session_msg_tx_map = session_msg_tx_map.clone();

        tokio::spawn(async move {
            on_client_connection(
                connection,
                task_message_converter,
                task_inbound_engine_message_tx,
                task_session_msg_tx_map,
            )
            .await;
        });
    }
}