mod message_conversion;

use crate::message_conversion::FixMessageConversion;
use common::engine::{InboundEngineMessage, NewOrderAck, OrderAction, OutboundEngineMessage, OutboundMessage, RejectionMessage};
use fefix::FixValue;
use lazy_static::lazy_static;
use std::error::Error;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Mutex};
use std::{env, thread};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

lazy_static! {
    pub static ref GATEWAY_PORT: u16 = env::var("GATEWAY_PORT").unwrap_or("3001".to_owned()).parse::<u16>().unwrap();
    pub static ref ENGINE_PORT: u16 = env::var("ENGINE_PORT").unwrap_or("3000".to_owned()).parse::<u16>().unwrap();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (inbound_engine_message_tx, inbound_engine_rx): (Sender<InboundEngineMessage>, Receiver<InboundEngineMessage>) = mpsc::channel();
    let fix = Arc::new(Mutex::new(FixMessageConversion::new()));

    let gateway_thread = tokio::spawn(async move {
        println!("Initializing FIX Gateway on port:{}", *GATEWAY_PORT);
        initialize_gateway_session_handler(fix, inbound_engine_message_tx).await.expect("failed to initialize gateway session");
    });

    let engine_submission_thread = thread::spawn(|| {
        println!("Initializing Engine message submission thread");
        initialize_engine_inbound_message_submission(inbound_engine_rx).expect("failed to initialize engine message submission thread");
    });

    engine_submission_thread.join().unwrap();
    gateway_thread.await.unwrap();
    Ok(())
}

async fn initialize_gateway_session_handler(fix_engine: Arc<Mutex<FixMessageConversion>>, inbound_engine_message_tx: Sender<InboundEngineMessage>) -> Result<(), Box<dyn Error>> {
    let tcp_listener = tokio::net::TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], *GATEWAY_PORT)))
        .await?;

    println!("Awaiting FIX session connections");


    loop {
        let (mut socket, _) = tcp_listener.accept().await?;

        let inbound_engine_message_tx = inbound_engine_message_tx.clone();
        let fix_arc_mutex = fix_engine.clone();

        tokio::spawn(async move {
            let mut in_buffer = vec![0; 2048];
            let mut out_buffer = vec![0; 2048];

            loop {
                let socket_bytes_read = socket
                    .read(&mut in_buffer)
                    .await
                    .expect("failed to read data from socket");

                if socket_bytes_read > 0 {
                    let inbound_message_parsed_result = fix_arc_mutex.lock()
                        .unwrap()
                        .fix_to_inbound_engine_message(&in_buffer[..socket_bytes_read]);


                    let outbound_engine_message: OutboundEngineMessage = match inbound_message_parsed_result {
                        Ok(message) => {
                            let inbound_engine_message = InboundEngineMessage {
                                seq_num: 0,
                                inbound_message: message,
                            };

                            let ss = inbound_engine_message_tx.send(inbound_engine_message);

                            OutboundEngineMessage {
                                seq_num: 0,
                                outbound_message: OutboundMessage::NewOrderAck(NewOrderAck {
                                    action: OrderAction::BUY,
                                    order_id: 0,
                                    px: 0,
                                    qty: 0,
                                    ack_time: 0,
                                }),
                            }
                        }
                        Err(_) => {
                            OutboundEngineMessage {
                                seq_num: 0,
                                outbound_message: OutboundMessage::RejectionMessage(RejectionMessage {
                                    reject_reason: 7
                                }),
                            }
                        }
                    };


                    let outbound_fix_message = fix_arc_mutex.lock()
                        .unwrap()
                        .outbound_engine_message_to_fix(outbound_engine_message);

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

fn initialize_engine_inbound_message_submission(rx: Receiver<InboundEngineMessage>) -> Result<(), Box<dyn Error>> {
    use socket2::{Domain, Type};
    let udp_multicast_socket = socket2::Socket::new(Domain::IPV4, Type::DGRAM, Some(socket2::Protocol::UDP)).expect("failed to create UDP socket");
    udp_multicast_socket.set_reuse_address(true).expect("failed to set reuse address");
    udp_multicast_socket.set_reuse_port(true).expect("failed to set reuse port");
    udp_multicast_socket.join_multicast_v4(&Ipv4Addr::new(239, 1, 1, 1), &Ipv4Addr::UNSPECIFIED).expect("failed to join multicast group");
    udp_multicast_socket.bind(&SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, *ENGINE_PORT).into()).expect("failed to bind UDP socket");

    let udp_socket = std::net::UdpSocket::from(udp_multicast_socket);
    let send_addr = "239.1.1.1:3000".parse::<SocketAddr>().unwrap();

    println!("Awaiting inbound messages");

    while let Ok(inbound_engine_message) = rx.recv() {
        let encoded: Vec<u8> = bitcode::encode(&inbound_engine_message);
        udp_socket.send_to(&encoded, send_addr).expect("TODO: panic message");
    }

    Ok(())
}