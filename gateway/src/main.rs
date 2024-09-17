mod fix_engine;

use crate::fix_engine::FixEngine;
use common::engine::{InboundEngineMessage, InboundMessage, NewOrder, OrderAction, OutboundEngineMessage, OutboundMessage, RejectionMessage};
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

    let (inbound_engine_message_tx, inbound_engine_rx): (Sender<InboundEngineMessage>, Receiver<InboundEngineMessage>) = mpsc::channel();

    let session_data_tx = Arc::new(Mutex::new(HashMap::<u32, Sender<OutboundEngineMessage>>::new()));
    let session_data_rx = Arc::new(Mutex::new(HashMap::<u32, Receiver<OutboundEngineMessage>>::new()));

    let engine_msg_in_thread = thread::spawn(|| {
        initialize_engine_in_message_submitter(inbound_engine_rx).expect("failed to initialize engine MSG_IN thread");
    });

    let session_data_tx_clone = session_data_tx.clone();
    let engine_msg_out_thread = thread::spawn(|| {
        initialize_engine_out_message_receiver(session_data_tx_clone).expect("failed to initialize engine MSG_OUT thread");
    });

    let session_data_tx_clone = session_data_tx.clone();
    initialize_gateway_session_handler(inbound_engine_message_tx, session_data_tx_clone, session_data_rx).await.expect("failed to initialize gateway session handler");

    engine_msg_out_thread.join().unwrap();
    engine_msg_in_thread.join().unwrap();
    Ok(())
}

async fn initialize_gateway_session_handler(inbound_engine_message_tx: Sender<InboundEngineMessage>, session_data_tx: Arc<Mutex<HashMap<u32, Sender<OutboundEngineMessage>>>>, session_data_rx: Arc<Mutex<HashMap<u32, Receiver<OutboundEngineMessage>>>>) -> Result<(), Box<dyn Error>> {
    let fix_engine = Arc::new(Mutex::new(FixEngine::new()));
    let tcp_listener = tokio::net::TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], *GATEWAY_PORT)))
        .await?;

    println!("Initialized Gateway FIX session handler {}", *GATEWAY_PORT);

    loop {
        let (mut socket, _) = tcp_listener.accept().await?;
        let task_fix_engine = fix_engine.clone();
        let task_session_data_tx = session_data_tx.clone();
        let task_session_data_rx = session_data_rx.clone();
        let task_inbound_engine_message_tx = inbound_engine_message_tx.clone();

        tokio::spawn(async move {
            // Client session
            // let session_id = random::<u32>();
            let session_id = 1;
            let mut session_fix_message_inbound_buffer = vec![0; 2048];

            // Engine MSG_OUT setup for client's current session
            let (outbound_engine_tx, outbound_engine_rx): (Sender<OutboundEngineMessage>, Receiver<OutboundEngineMessage>) = mpsc::channel();
            task_session_data_tx.lock().unwrap().insert(session_id, outbound_engine_tx);
            // task_session_data_rx.lock().unwrap().insert(session_id, outbound_engine_rx);


            loop {
                let socket_bytes_read = socket
                    .read(&mut session_fix_message_inbound_buffer)
                    .await
                    .expect("failed to read data from socket");

                if socket_bytes_read > 0 {
                    let inbound_message_parsed_result = task_fix_engine.lock()
                        .unwrap()
                        .fix_to_inbound_engine_message(&session_fix_message_inbound_buffer[..socket_bytes_read]);

                    let buy = InboundEngineMessage {
                        seq_num: 0,
                        inbound_message: InboundMessage::NewOrder(NewOrder {
                            order_action: OrderAction::BUY,
                            px: 50,
                            qty: 100,
                        }),
                    };

                    let sell = InboundEngineMessage {
                        seq_num: 0,
                        inbound_message: InboundMessage::NewOrder(NewOrder {
                            order_action: OrderAction::SELL,
                            px: 50,
                            qty: 100,
                        }),
                    };

                    task_inbound_engine_message_tx.send(buy).unwrap();
                    task_inbound_engine_message_tx.send(sell).unwrap();

                    loop {
                        match outbound_engine_rx.try_recv() {
                            Ok(engine_out_message) => {
                                println!("Received message for client from engine {:?}", engine_out_message);
                            }
                            Err(_) => {}
                        }
                    }

                    let outbound_fix_message = task_fix_engine.lock()
                        .unwrap()
                        .outbound_engine_message_to_fix(OutboundEngineMessage {
                            session_id,
                            seq_num: 0,
                            outbound_message: OutboundMessage::RejectionMessage(RejectionMessage {
                                reject_reason: 0,
                            }),
                        });

                    socket
                        .write_all(&outbound_fix_message)
                        .await
                        .expect("failed to write data to socket");
                } else {
                    return;
                }
            }
        });
    }
}

fn initialize_engine_in_message_submitter(rx: Receiver<InboundEngineMessage>) -> Result<(), Box<dyn Error>> {
    use socket2::{Domain, Type};
    let udp_multicast_socket = socket2::Socket::new(Domain::IPV4, Type::DGRAM, Some(socket2::Protocol::UDP)).expect("failed to create UDP socket");
    udp_multicast_socket.set_reuse_address(true).expect("failed to set reuse address");
    udp_multicast_socket.set_reuse_port(true).expect("failed to set reuse port");

    let udp_socket = std::net::UdpSocket::from(udp_multicast_socket);
    let send_addr = "0.0.0.0:3000".parse::<SocketAddr>().unwrap();

    println!("Initialized Gateway -> MSG_IN multicast on port {}", * ENGINE_MSG_IN_PORT);

    while let Ok(inbound_engine_message) = rx.recv() {
        let encoded: Vec<u8> = bitcode::encode(&inbound_engine_message);
        udp_socket.send_to(&encoded, send_addr).expect("TODO: panic message");
    }

    Ok(())
}

fn initialize_engine_out_message_receiver(session_data_tx: Arc<Mutex<HashMap<u32, Sender<OutboundEngineMessage>>>>) -> Result<(), Box<dyn Error>> {
    use socket2::{Domain, Type};
    let udp_multicast_socket = socket2::Socket::new(Domain::IPV4, Type::DGRAM, Some(socket2::Protocol::UDP)).expect("failed to create UDP socket");
    udp_multicast_socket.set_reuse_address(true).expect("failed to set reuse address");
    udp_multicast_socket.set_reuse_port(true).expect("failed to set reuse port");
    udp_multicast_socket.bind(&SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, *ENGINE_MSG_OUT_PORT).into()).expect("failed to bind UDP socket");

    let udp_socket = std::net::UdpSocket::from(udp_multicast_socket);

    let mut buffer = [0; 64000];

    println!("Initialized MSG_OUT -> Gateway multicast on port {}", * ENGINE_MSG_OUT_PORT);

    loop {
        match udp_socket.recv_from(&mut buffer) {
            Ok((size, addr)) => {
                let outbound_engine_messages: OutboundEngineMessage = bitcode::decode(&buffer[..size]).unwrap();

                match session_data_tx.lock() {
                    Ok(session_data) => {
                        let blah = session_data.get(&outbound_engine_messages.session_id).unwrap();
                        blah.send(outbound_engine_messages).unwrap()
                    }
                    Err(_) => {}
                }

            }
            Err(_) => {}
        }
    }
    Ok(())
}

