mod fix_engine;

use crate::fix_engine::FixEngine;
use common::engine::{InboundEngineMessage, InboundMessage, NewOrderAck, OrderAction, OutboundEngineMessage, OutboundMessage, RejectionMessage};
use fefix::FixValue;
use lazy_static::lazy_static;
use std::error::Error;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Mutex};
use std::{env, thread};
use std::collections::HashMap;
use rand::random;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

lazy_static! {
    pub static ref GATEWAY_PORT: u16 = env::var("GATEWAY_PORT").unwrap_or("3001".to_owned()).parse::<u16>().unwrap();
    pub static ref ENGINE_MSG_IN_PORT: u16 = env::var("ENGINE_PORT").unwrap_or("3000".to_owned()).parse::<u16>().unwrap();
    pub static ref ENGINE_MSG_OUT_PORT: u16 = env::var("ENGINE_PORT").unwrap_or("3500".to_owned()).parse::<u16>().unwrap();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (inbound_engine_message_tx, inbound_engine_rx): (Sender<InboundEngineMessage>, Receiver<InboundEngineMessage>) = mpsc::channel();
    let engine_out_message_store = Arc::new(Mutex::new(HashMap::<u32, Vec<OutboundEngineMessage>>::new()));

    let gateway_client_session_message_store = engine_out_message_store.clone();
    let gateway_client_session_message_store1 = engine_out_message_store.clone();

    let engine_msg_in_thread = thread::spawn(|| {
        println!("Initializing Engine MSG_IN thread");
        initialize_engine_in_message_submitter(inbound_engine_rx).expect("failed to initialize engine MSG_IN thread");
    });

    let engine_msg_out_thread = thread::spawn(|| {
        println!("Initializing Engine MSG_OUT thread");
        initialize_engine_out_message_receiver(gateway_client_session_message_store).expect("failed to initialize engine MSG_OUT thread");
    });


    initialize_gateway_session_handler(inbound_engine_message_tx, gateway_client_session_message_store1).await.expect("failed to initialize gateway session handler");

    engine_msg_out_thread.join().unwrap();
    engine_msg_in_thread.join().unwrap();
    Ok(())
}

async fn initialize_gateway_session_handler(inbound_engine_message_tx: Sender<InboundEngineMessage>, session_message_store: Arc<Mutex<HashMap<u32, Vec<OutboundEngineMessage>>>>) -> Result<(), Box<dyn Error>> {
    let fix_engine = Arc::new(Mutex::new(FixEngine::new()));
    let tcp_listener = tokio::net::TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], *GATEWAY_PORT)))
        .await?;

    println!("Awaiting FIX session connections");

    loop {
        let (mut socket, _) = tcp_listener.accept().await?;

        let inbound_engine_message_tx = inbound_engine_message_tx.clone();
        let fix_arc_mutex = fix_engine.clone();

        tokio::spawn(async move {

            let mut in_buffer = vec![0; 2048];
            let session_id = random::<u32>();
            let mut seq_no = 0;


            loop {

                let socket_bytes_read = socket
                    .read(&mut in_buffer)
                    .await
                    .expect("failed to read data from socket");

                if socket_bytes_read > 0 {
                    let inbound_message_parsed_result = fix_arc_mutex.lock()
                        .unwrap()
                        .fix_to_inbound_engine_message(&in_buffer[..socket_bytes_read]);

                    let inbound_engine_message = InboundEngineMessage {
                        seq_num: seq_no,
                        inbound_message: inbound_message_parsed_result.unwrap(),
                    };

                    let ss = inbound_engine_message_tx.send(inbound_engine_message);




                    let outbound_fix_message = fix_arc_mutex.lock()
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
    udp_multicast_socket.join_multicast_v4(&Ipv4Addr::new(239, 1, 1, 1), &Ipv4Addr::UNSPECIFIED).expect("failed to join multicast group");
    udp_multicast_socket.bind(&SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, *ENGINE_MSG_IN_PORT).into()).expect("failed to bind UDP socket");

    let udp_socket = std::net::UdpSocket::from(udp_multicast_socket);
    let send_addr = "239.1.1.1:3000".parse::<SocketAddr>().unwrap();

    println!("Awaiting GATEWAY messages");

    while let Ok(inbound_engine_message) = rx.recv() {
        let encoded: Vec<u8> = bitcode::encode(&inbound_engine_message);
        udp_socket.send_to(&encoded, send_addr).expect("TODO: panic message");
    }

    Ok(())
}

fn initialize_engine_out_message_receiver(engine_out_message_store: Arc<Mutex<HashMap<u32, Vec<OutboundEngineMessage>>>>) -> Result<(), Box<dyn Error>> {
    use socket2::{Domain, Type};
    let udp_multicast_socket = socket2::Socket::new(Domain::IPV4, Type::DGRAM, Some(socket2::Protocol::UDP)).expect("failed to create UDP socket");
    udp_multicast_socket.set_reuse_address(true).expect("failed to set reuse address");
    udp_multicast_socket.set_reuse_port(true).expect("failed to set reuse port");
    udp_multicast_socket.bind(&SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, *ENGINE_MSG_OUT_PORT).into()).expect("failed to bind UDP socket");

    let udp_socket = std::net::UdpSocket::from(udp_multicast_socket);

    let mut buffer = [0; 64000];
    let mut req_per_second: usize = 0;
    let mut time = minstant::Instant::now();

    println!("Awaiting engine MSG_OUT messages");

    let mut seq = 0;
    loop {
        let (size, addr) = udp_socket.recv_from(&mut buffer).unwrap();
        let outbound_engine_message: Vec<OutboundEngineMessage> = bitcode::decode(&buffer[..size]).unwrap();

        // seq = outbound_engine_message.seq_num;

        if time.elapsed().as_millis() > 1000 {
            time = minstant::Instant::now();
            println!("Msg / sec: {}", req_per_second * 4096);
            req_per_second = 0;
        }

        req_per_second += 1;
    }
    Ok(())
}

