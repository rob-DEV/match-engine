use crate::parser::MessageConverter;
use common::domain::messaging::{EngineMessage};
use rand::random;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::Interval;
use crate::message::GatewayMessage;

struct ClientSessionState {
    client_id: u32,
    session_ip: SocketAddr,
    heartbeat_interval: Interval,

    initial_seq_number: u32,
    last_seen_client_seq_number: u32,
    last_seen_gateway_seq_number: u32,

    logged_in: bool,

    messages: Vec<String>,
}

impl ClientSessionState {
    pub fn new(addr: SocketAddr) -> ClientSessionState {
        let client_id = random::<u32>();

        ClientSessionState {
            client_id,
            session_ip: addr,
            heartbeat_interval: tokio::time::interval(Duration::from_millis(300)),
            initial_seq_number: 0,
            last_seen_client_seq_number: 0,
            last_seen_gateway_seq_number: 0,
            logged_in: false,
            messages: vec![],
        }
    }
}

pub async fn on_client_connection(connection: (TcpStream, SocketAddr), message_converter: Arc<Mutex<MessageConverter>>, inbound_engine_message_tx: Sender<GatewayMessage>, client_msg_tx_map: Arc<Mutex<HashMap<u32, Sender<EngineMessage>>>>) {
    let (mut stream, client_addr) = connection;
    let (reader, writer) = stream.into_split();
    let mut buf_reader = BufReader::new(reader);
    let mut writer = writer;

    let client_session_state = ClientSessionState::new(client_addr);

    let (send_tx, mut receiver_rx): (Sender<EngineMessage>, Receiver<EngineMessage>) = mpsc::channel();

    let ss = client_msg_tx_map.clone();

    ss.lock().unwrap().insert(client_session_state.client_id, send_tx);

    let read_task = tokio::spawn(async move {
        let mut line = String::new();
        loop {
            line.clear();
            let bytes_read = buf_reader.read_line(&mut line).await.unwrap();

            let inbound_client_message = message_converter.lock().unwrap().fix_to_in_msg(client_session_state.client_id, &line.as_bytes()[..bytes_read - 1]);

            match inbound_client_message {
                Ok(inbound_message) => { inbound_engine_message_tx.clone().send(inbound_message).unwrap() }
                Err(err) => { eprintln!("Error: {}", err); }
            }

            if bytes_read == 0 {
                println!("Client disconnected");
                break;
            }
        }
    });

    let write_task = tokio::spawn(async move {
        while let Ok(outbound) = receiver_rx.recv() {
            writer.write_all(format!("{:?}\n", outbound).as_bytes()).await.unwrap();
        }
    });

    let _ = tokio::try_join!(read_task, write_task);
}