use crate::fix_engine::MessageConverter;
use common::engine::{InboundEngineMessage, InboundMessage, OutboundEngineMessage, OutboundMessage, RejectionMessage};
use fefix::FixValue;
use rand::random;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
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
        }
    }
}

pub async fn handle_fix_session(connection: (TcpStream, SocketAddr), message_converter: Arc<Mutex<MessageConverter>>, inbound_engine_message_tx: Sender<InboundEngineMessage>, session_msg_tx_map: Arc<Mutex<HashMap<u32, Sender<OutboundEngineMessage>>>>) {
    let (mut socket, client_addr) = connection;
    let (mut read, mut write) = socket.split();

    let mut state = FixSessionState::new(client_addr, inbound_engine_message_tx, session_msg_tx_map);

    let mut fix_message_inbound_buffer = vec![0; 2048];

    loop {
        tokio::select! {
            _ = state.heartbeat_interval.tick() => {
                write.write(&"heartbeat-message\n".to_bytes()).await;
            }

            result = read.read(&mut fix_message_inbound_buffer) => {
                let _bytes_read = result.unwrap();

                 let inbound_message_result = message_converter.lock()
                        .unwrap()
                        .fix_to_in_msg(&fix_message_inbound_buffer[.._bytes_read]);

                match inbound_message_result {
                    Ok(engine_msg_in) => {
                        match engine_msg_in.inbound_message {
                            InboundMessage::Logon(logon) => { write.write(&fix_message_inbound_buffer).await; state.logged_in = true; },
                            InboundMessage::LogOut(logout) => { write.write(&fix_message_inbound_buffer).await; }
                            InboundMessage::NewOrder(new_order) => {}
                            InboundMessage::CancelOrder(cancel_order) => {}
                        }
                    }
                    Err(decode_err) => {
                        // Reject inbound fix // apply seq nonetheless
                        let rejection = message_converter.lock()
                        .unwrap()
                        .engine_msg_out_to_fix(OutboundEngineMessage {
                            session_id: state.session_id,
                            seq_num: 0,
                            outbound_message: OutboundMessage::RejectionMessage(RejectionMessage {
                                reject_reason: 0,
                            }),
                        });
                        write.write(&rejection).await;
                    }
                }

                write.write(&fix_message_inbound_buffer).await;
            }
        }
    }
}