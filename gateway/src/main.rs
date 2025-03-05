mod parser;
mod client_state;
mod message;

use crate::client_state::on_client_connection;
use crate::message::GatewayMessage;
use crate::parser::MessageConverter;
use common::domain::domain::TradeExecution;
use common::domain::messaging::{EngineMessage, SequencedEngineMessage};
use common::network::udp_socket::multicast_udp_socket;
use fefix::FixValue;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Mutex};
use std::{env, thread};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

lazy_static! {
    pub static ref GATEWAY_PORT: u16 = env::var("GATEWAY_PORT").unwrap_or("3001".to_owned()).parse::<u16>().unwrap();
    pub static ref ENGINE_MSG_IN_PORT: u16 = env::var("ENGINE_PORT").unwrap_or("3000".to_owned()).parse::<u16>().unwrap();
    pub static ref ENGINE_MSG_OUT_PORT: u16 = env::var("ENGINE_PORT").unwrap_or("3500".to_owned()).parse::<u16>().unwrap();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("--- Initializing Gateway ---");

    let (client_msg_tx, client_msg_rx): (Sender<GatewayMessage>, Receiver<GatewayMessage>) = mpsc::channel();
    let (msg_in_tx, engine_msg_in_rx): (Sender<SequencedEngineMessage>, Receiver<SequencedEngineMessage>) = mpsc::channel();
    let gateway_to_engine_msg_in_tx = Arc::new(Mutex::new(HashMap::<u32, Sender<EngineMessage>>::new()));

    let engine_msg_in_thread = thread::spawn(|| {
        initialize_msg_in_message_submitter(client_msg_rx).expect("failed to initialize engine MSG_IN thread");
    });

    let thread_session_msg_tx_map = gateway_to_engine_msg_in_tx.clone();
    let engine_msg_out_thread = thread::spawn(|| {
        initialize_engine_msg_out_receiver(thread_session_msg_tx_map).expect("failed to initialize engine MSG_OUT thread");
    });

    let thread_session_msg_tx_map = gateway_to_engine_msg_in_tx.clone();
    initialize_gateway_session_handler(client_msg_tx, thread_session_msg_tx_map).await.expect("failed to initialize gateway session handler");

    engine_msg_out_thread.join().unwrap();
    engine_msg_in_thread.join().unwrap();
    Ok(())
}

async fn initialize_gateway_session_handler(inbound_engine_message_tx: Sender<GatewayMessage>, session_msg_tx_map: Arc<Mutex<HashMap<u32, Sender<EngineMessage>>>>) -> Result<(), Box<dyn Error>> {
    // Shared for now
    let message_converter = Arc::new(Mutex::new(MessageConverter::new()));
    let tcp_listener = tokio::net::TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], *GATEWAY_PORT)))
        .await?;

    println!("Initialized Gateway FIX session handler {}", *GATEWAY_PORT);

    loop {
        let connection = tcp_listener.accept().await?;
        let task_message_converter = message_converter.clone();
        let task_inbound_engine_message_tx = inbound_engine_message_tx.clone();
        let task_session_msg_tx_map = session_msg_tx_map.clone();

        tokio::spawn(async move {
            on_client_connection(connection, task_message_converter, task_inbound_engine_message_tx, task_session_msg_tx_map).await;
        });
    }
}

fn initialize_msg_in_message_submitter(rx: Receiver<GatewayMessage>) -> Result<(), Box<dyn Error>> {
    let udp_socket = multicast_udp_socket(*ENGINE_MSG_IN_PORT, false);
    let send_addr = "0.0.0.0:3000".parse::<SocketAddr>().unwrap();
    println!("Initialized Gateway -> MSG_IN multicast on port {}", *ENGINE_MSG_IN_PORT);

    let mut sequence = 1;
    loop {
        while let Ok(inbound_engine_message) = rx.recv() {
            // let mut r = Vec::new();

            let message_in = match inbound_engine_message {
                GatewayMessage::LimitOrder(new) => {
                    // let s = random::<u8>();
                    // let sc = random::<u32>();
                    // let sx = random::<u32>();
                    //
                    // for i in 0..1 {
                    //     r.push(NewOrder {
                    //         client_id: new.client_id,
                    //         order_action: if s % 2 == 0 { BUY } else { SELL },
                    //         px: sc % 100,
                    //         qty: sx % 10,
                    //         timestamp: epoch_nanos(),
                    //     })
                    // }

                    SequencedEngineMessage {
                        sequence_number: sequence,
                        message: EngineMessage::NewOrder(new),
                    }
                }
                GatewayMessage::MarketOrder(_) => {
                    unimplemented!()
                }
                GatewayMessage::CancelOrder(cancel) => {
                    SequencedEngineMessage {
                        sequence_number: sequence,
                        message: EngineMessage::CancelOrder(cancel),
                    }
                }
            };
            let encoded: Vec<u8> = bitcode::encode(&message_in);
            udp_socket.send_to(&encoded, send_addr).expect("TODO: panic message");

            let mut ack_bits = [0u8; 4];
            udp_socket.recv_from(&mut ack_bits).expect("TODO: panic message");

            let id: u32 = u32::from_le_bytes([ack_bits[0], ack_bits[1], ack_bits[2], ack_bits[3]]);
            assert_eq!(id, sequence);

            sequence += 1;
        };
    }
}

fn initialize_engine_msg_out_receiver(session_data_tx: Arc<Mutex<HashMap<u32, Sender<EngineMessage>>>>) -> Result<(), Box<dyn Error>> {
    let udp_socket = multicast_udp_socket(*ENGINE_MSG_OUT_PORT, true);
    let mut buffer = [0; 256];

    println!("Initialized MSG_OUT -> Gateway multicast on port {}", *ENGINE_MSG_OUT_PORT);

    loop {
        match udp_socket.recv_from(&mut buffer) {
            Ok((size, _)) => {
                let outbound_engine_message: SequencedEngineMessage = bitcode::decode(&buffer[..size]).unwrap();

                let outbound_message_type = &outbound_engine_message.message;

                let mut session_data = session_data_tx.lock().unwrap();
                match outbound_message_type {
                    EngineMessage::NewOrderAck(new_ack) => {
                        let client_id = new_ack.client_id;

                        let session_state = session_data.get_mut(&client_id).unwrap();
                        session_state.send(outbound_engine_message.message).unwrap()
                    }
                    EngineMessage::CancelOrderAck(cancel_ack) => {
                        let client_id = cancel_ack.client_id;
                        let session_state = session_data.get_mut(&client_id).unwrap();
                        session_state.send(outbound_engine_message.message).unwrap()
                    }
                    EngineMessage::TradeExecution(execution) => {
                        let bid_client_id = execution.bid_client_id;
                        let ask_client_id = execution.ask_client_id;

                        let ask_exec = EngineMessage::TradeExecution(TradeExecution {
                            trade_id: execution.trade_id,
                            trade_seq: execution.trade_seq,
                            bid_client_id: execution.bid_client_id,
                            ask_client_id: execution.ask_client_id,
                            bid_order_id: execution.bid_order_id,
                            ask_order_id: execution.ask_order_id,
                            fill_qty: execution.fill_qty,
                            px: execution.px,
                            execution_time: execution.execution_time,
                        });

                        let bid_tx = session_data.get(&bid_client_id).unwrap();
                        bid_tx.send(outbound_engine_message.message).unwrap();

                        let ask_tx = session_data.get(&ask_client_id).unwrap();
                        ask_tx.send(ask_exec).unwrap();
                    }
                    _ => { unimplemented!() }
                }
            }
            Err(_) => {}
        }
    }

    Ok(())
}

