mod fix_engine;
mod fix_server;

use crate::fix_engine::MessageConverter;
use crate::fix_server::on_client_connection;
use common::drain::rx_drain_with_timeout;
use common::messaging::TradeExecution;
use common::transport::{EngineMessage, GatewayMessage, InboundEngineMessage, OutboundEngineMessage};
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

    let (client_msg_tx, client_msg_rx): (Sender<GatewayMessage>, Receiver<GatewayMessage>) = mpsc::channel();
    let (msg_in_tx, engine_msg_in_rx): (Sender<InboundEngineMessage>, Receiver<InboundEngineMessage>) = mpsc::channel();
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
    use socket2::{Domain, Type};
    let udp_multicast_socket = socket2::Socket::new(Domain::IPV4, Type::DGRAM, Some(socket2::Protocol::UDP)).expect("failed to create UDP socket");
    udp_multicast_socket.set_reuse_address(true).expect("failed to set reuse address");
    udp_multicast_socket.set_reuse_port(true).expect("failed to set reuse port");

    let udp_socket = std::net::UdpSocket::from(udp_multicast_socket);
    let send_addr = "0.0.0.0:3000".parse::<SocketAddr>().unwrap();
    println!("Initialized Gateway -> MSG_IN multicast on port {}", *ENGINE_MSG_IN_PORT);

    let mut msg_buff = Vec::<InboundEngineMessage>::with_capacity(128);

    loop {
        let drained = rx_drain_with_timeout::<GatewayMessage, InboundEngineMessage>(&rx, &mut msg_buff, |msg| {
            match msg {
                GatewayMessage::Logon(_) => !unimplemented!(),
                GatewayMessage::LogOut(_) => !unimplemented!(),
                GatewayMessage::NewOrder(new_order) => {
                    InboundEngineMessage {
                        sequence_number: 0,
                        message: EngineMessage::NewOrder(new_order),
                    }
                }
            }
        }, 5000);

        if drained > 0 {
            println!("Order batch {}", msg_buff.len());
            let encoded: Vec<u8> = bitcode::encode(&msg_buff);
            udp_socket.send_to(&encoded, send_addr).expect("TODO: panic message");
            msg_buff.clear();
        }
    }
}

fn initialize_engine_msg_out_receiver(session_data_tx: Arc<Mutex<HashMap<u32, Sender<EngineMessage>>>>) -> Result<(), Box<dyn Error>> {
    use socket2::{Domain, Type};
    let udp_multicast_socket = socket2::Socket::new(Domain::IPV4, Type::DGRAM, Some(socket2::Protocol::UDP)).expect("failed to create UDP socket");
    udp_multicast_socket.set_reuse_address(true).expect("failed to set reuse address");
    udp_multicast_socket.set_reuse_port(true).expect("failed to set reuse port");
    udp_multicast_socket.bind(&SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, *ENGINE_MSG_OUT_PORT).into()).expect("failed to bind UDP socket");

    let udp_socket = std::net::UdpSocket::from(udp_multicast_socket);

    let mut buffer = [0; 32768];

    println!("Initialized MSG_OUT -> Gateway multicast on port {}", *ENGINE_MSG_OUT_PORT);

    loop {
        match udp_socket.recv_from(&mut buffer) {
            Ok((size, _)) => {
                let outbound_engine_message: OutboundEngineMessage = bitcode::decode(&buffer[..size]).unwrap();
                // println!("MSG_OUT {:?}", outbound_engine_message);

                let outbound_message_type = &outbound_engine_message.message;

                let mut session_data = session_data_tx.lock().unwrap();
                match outbound_message_type {
                    EngineMessage::NewOrderAck(a) => {
                        let client_id = a.client_id;

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

