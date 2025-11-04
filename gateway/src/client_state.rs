use crate::message::GatewayMessage;
use crate::parser::MessageConverter;
use dashmap::DashMap;
use rand::random;
use std::net::SocketAddr;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use common::transport::sequenced_message::EngineMessage;

struct ClientSessionState {
    client_id: u32,
}

impl ClientSessionState {
    pub fn new() -> ClientSessionState {
        let client_id = random::<u32>();

        ClientSessionState { client_id }
    }
}

pub async fn on_client_connection(
    connection: (TcpStream, SocketAddr),
    inbound_engine_message_tx: Sender<GatewayMessage>,
    client_msg_tx_map: Arc<DashMap<u32, Sender<EngineMessage>>>,
) {
    println!("Client connected!");
    let (stream, client_addr) = connection;
    let (reader, writer) = stream.into_split();
    let mut buf_reader = BufReader::new(reader);
    let mut writer = writer;

    let client_session_state = ClientSessionState::new();

    let (send_tx, receiver_rx): (Sender<EngineMessage>, Receiver<EngineMessage>) = mpsc::channel();

    let client_message_out_map = client_msg_tx_map.clone();

    client_message_out_map.insert(client_session_state.client_id, send_tx);

    let mut message_parser = MessageConverter::new();

    let read_task = tokio::spawn(async move {
        let mut line = String::new();
        let mut engine_message_tx = inbound_engine_message_tx.clone();
        loop {
            line.clear();
            let bytes_read = buf_reader.read_line(&mut line).await.unwrap();

            let inbound_client_message = message_parser.fix_to_in_msg(
                client_session_state.client_id,
                &line.as_bytes()[..bytes_read - 1],
            );

            match inbound_client_message {
                Ok(inbound_message) => engine_message_tx
                    .send(inbound_message)
                    .unwrap(),
                Err(err) => {
                    eprintln!("Error: {}", err);
                }
            }

            if bytes_read == 0 {
                println!("Client disconnected");
                break;
            }
        }
    });

    let write_task = tokio::spawn(async move {
        while let Ok(outbound) = receiver_rx.recv() {
            writer
                .write_all(format!("{:?}\n", outbound).as_bytes())
                .await
                .unwrap();
        }
    });

    let _ = tokio::try_join!(read_task, write_task);
}
