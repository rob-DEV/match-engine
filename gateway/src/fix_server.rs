use crate::fix_engine::MessageConverter;
use common::engine::{InboundEngineMessage, OutboundEngineMessage};
use rand::random;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::Interval;

struct FixSessionState {
    session_id: u32,
    session_ip: SocketAddr,
    heartbeat_interval: Interval,

    initial_seq_number: u32,
    last_seen_client_seq_number: u32,
    last_seen_gateway_seq_number: u32,

    logged_in: bool,

    engine_msg_in_tx: Sender<InboundEngineMessage>,
    engine_msg_out_rx: Receiver<OutboundEngineMessage>,
    messages: Vec<String>,
}

impl FixSessionState {
    pub fn new(addr: SocketAddr, engine_msg_in_tx: Sender<InboundEngineMessage>, session_msg_tx_map: Arc<Mutex<HashMap<u32, Sender<OutboundEngineMessage>>>>) -> FixSessionState {
        let session_id = random::<u32>();
        let (engine_msg_out_tx, engine_msg_out_rx) = mpsc::channel::<OutboundEngineMessage>();

        session_msg_tx_map.lock().unwrap().insert(session_id, engine_msg_out_tx);

        FixSessionState {
            session_id,
            session_ip: addr,
            heartbeat_interval: tokio::time::interval(Duration::from_millis(300)),
            initial_seq_number: 0,
            last_seen_client_seq_number: 0,
            last_seen_gateway_seq_number: 0,
            logged_in: false,
            engine_msg_in_tx,
            engine_msg_out_rx,
            messages: vec![],
        }
    }
}

pub async fn on_client_connection(connection: (TcpStream, SocketAddr), message_converter: Arc<Mutex<MessageConverter>>, inbound_engine_message_tx: Sender<InboundEngineMessage>, session_msg_tx_map: Arc<Mutex<HashMap<u32, Sender<OutboundEngineMessage>>>>) {
    let (mut stream, client_addr) = connection;
    let (reader, writer) = stream.into_split();
    let mut buf_reader = BufReader::new(reader);
    let mut writer = writer;

    let read_task = tokio::spawn(async move {
        let mut line = String::new();
        loop {
            line.clear();
            let bytes_read = buf_reader.read_line(&mut line).await.unwrap();

            let fix = message_converter.lock().unwrap().fix_to_in_msg(&line.as_bytes()[..bytes_read - 1]);

            match fix {
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
        let mut count = 0;
        loop {
            // let message = format!("Message number: {}\n", count);
            // if writer.write_all(message.as_bytes()).await.is_err() {
            //     break;
            // }
            // count += 1;
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });

    let _ = tokio::try_join!(read_task, write_task);
}