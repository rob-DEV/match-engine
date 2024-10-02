mod fix_engine;
mod fix_server;

use crate::fix_engine::MessageConverter;
use crate::fix_server::on_client_connection;
use common::engine::{InboundEngineMessage, InboundMessage, OutboundEngineMessage, OutboundMessage, TradeExecution};
use fefix::FixValue;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::error::Error;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
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

    let (msg_in_tx, engine_msg_in_rx): (Sender<InboundMessage>, Receiver<InboundMessage>) = mpsc::channel();
    let gateway_to_engine_msg_in_tx = Arc::new(Mutex::new(HashMap::<u32, Arc<Mutex<Sender<OutboundMessage>>>>::new()));

    let engine_msg_in_thread = thread::spawn(|| {
        initialize_msg_in_message_submitter(engine_msg_in_rx).expect("failed to initialize engine MSG_IN thread");
    });

    let thread_session_msg_tx_map = gateway_to_engine_msg_in_tx.clone();
    let engine_msg_out_thread = thread::spawn(|| {
        initialize_engine_msg_out_receiver(thread_session_msg_tx_map).expect("failed to initialize engine MSG_OUT thread");
    });

    let thread_session_msg_tx_map = gateway_to_engine_msg_in_tx.clone();
    initialize_gateway_session_handler(msg_in_tx, thread_session_msg_tx_map).await.expect("failed to initialize gateway session handler");

    engine_msg_out_thread.join().unwrap();
    engine_msg_in_thread.join().unwrap();
    Ok(())
}

async fn initialize_gateway_session_handler(inbound_engine_message_tx: Sender<InboundMessage>, session_msg_tx_map: Arc<Mutex<HashMap<u32, Arc<Mutex<Sender<OutboundMessage>>>>>>) -> Result<(), Box<dyn Error>> {
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

fn initialize_msg_in_message_submitter(rx: Receiver<InboundMessage>) -> Result<(), Box<dyn Error>> {
    use socket2::{Domain, Type};
    let udp_multicast_socket = socket2::Socket::new(Domain::IPV4, Type::DGRAM, Some(socket2::Protocol::UDP)).expect("failed to create UDP socket");
    udp_multicast_socket.set_reuse_address(true).expect("failed to set reuse address");
    udp_multicast_socket.set_reuse_port(true).expect("failed to set reuse port");

    let udp_socket = std::net::UdpSocket::from(udp_multicast_socket);
    let send_addr = "0.0.0.0:3000".parse::<SocketAddr>().unwrap();

    println!("Initialized Gateway -> MSG_IN multicast on port {}", *ENGINE_MSG_IN_PORT);

    let mut in_msg_seq_num = 0;

    while let Ok(inbound_engine_message) = rx.recv() {
        let inbound_engine_message = InboundEngineMessage {
            seq_num: in_msg_seq_num,
            inbound_message: inbound_engine_message,
        };

        let encoded: Vec<u8> = bitcode::encode(&inbound_engine_message);
        udp_socket.send_to(&encoded, send_addr).expect("TODO: panic message");

        in_msg_seq_num += 1;
    }

    Ok(())
}

fn initialize_engine_msg_out_receiver(session_data_tx: Arc<Mutex<HashMap<u32, Arc<Mutex<Sender<OutboundMessage>>>>>>) -> Result<(), Box<dyn Error>> {
    use socket2::{Domain, Type};
    let udp_multicast_socket = socket2::Socket::new(Domain::IPV4, Type::DGRAM, Some(socket2::Protocol::UDP)).expect("failed to create UDP socket");
    udp_multicast_socket.set_reuse_address(true).expect("failed to set reuse address");
    udp_multicast_socket.set_reuse_port(true).expect("failed to set reuse port");
    udp_multicast_socket.bind(&SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, *ENGINE_MSG_OUT_PORT).into()).expect("failed to bind UDP socket");

    let udp_socket = std::net::UdpSocket::from(udp_multicast_socket);

    let mut buffer = [0; 64000];

    println!("Initialized MSG_OUT -> Gateway multicast on port {}", *ENGINE_MSG_OUT_PORT);

    loop {
        match udp_socket.recv_from(&mut buffer) {
            Ok((size, _)) => {
                let outbound_engine_message: OutboundEngineMessage = bitcode::decode(&buffer[..size]).unwrap();
                println!("MSG_OUT {:?}", outbound_engine_message);

                let outbound_message_type = &outbound_engine_message.outbound_message;

                let mut lock = session_data_tx.lock().unwrap();
                match outbound_message_type {
                    OutboundMessage::NewOrderAck(a) => {
                        let client_id = a.client_id;

                        let blah = lock.get_mut(&client_id).unwrap().lock().unwrap();
                        blah.send(outbound_engine_message.outbound_message).unwrap()
                    }
                    OutboundMessage::TradeExecution(execution) => {
                        let bid_client_id = execution.bid_client_id;
                        let ask_client_id = execution.ask_client_id;

                        let ask_exec = OutboundMessage::TradeExecution(TradeExecution {
                            execution_id: execution.execution_id,
                            bid_client_id: execution.bid_client_id,
                            bid_id: execution.bid_id,
                            ask_client_id,
                            ask_id: execution.ask_id,
                            fill_qty: execution.fill_qty,
                            px: execution.px,
                            execution_time: execution.execution_time,
                        });

                        let blah = lock.get(&bid_client_id).unwrap().lock().unwrap();
                        blah.send(outbound_engine_message.outbound_message).unwrap();

                        // hack clone to other side
                        let blah = lock.get(&ask_client_id).unwrap().lock().unwrap();
                        blah.send(ask_exec).unwrap();
                    }
                    _ => { unimplemented!() }
                }
            }
            Err(_) => {}
        }
    }
    Ok(())
}

