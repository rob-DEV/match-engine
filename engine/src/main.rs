use crate::config::config::load_engine_config;
use crate::internal::order::{CancelOrder, LimitOrder, Order};
use common::multicast::multicast_udp_socket;
use common::time::{epoch_nanos, wait_50_millis};
use lazy_static::lazy_static;
use rand::random;
use std::error::Error;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::{env, thread};
use common::transport::{EngineMessage, InboundEngineMessage, OutboundEngineMessage};

mod engine;
mod internal;
mod util;
mod config;
mod book;

lazy_static! {
    pub static ref ENGINE_MSG_IN_PORT: u16 = env::var("ENGINE_PORT").unwrap_or("3000".to_owned()).parse::<u16>().unwrap();
    pub static ref ENGINE_MSG_OUT_PORT: u16 = env::var("ENGINE_PORT").unwrap_or("3500".to_owned()).parse::<u16>().unwrap();
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("--- Initializing Match Engine ---");
    let config = load_engine_config();

    let (engine_msg_out_tx, engine_msg_out_rx): (Sender<OutboundEngineMessage>, Receiver<OutboundEngineMessage>) = mpsc::channel();
    let (order_entry_tx, order_entry_rx): (Sender<Order>, Receiver<Order>) = mpsc::channel();

    let engine_thread = thread::spawn(move || {
        let mut match_engine = engine::match_engine::MatchEngine::new(config.get("symbol").unwrap().to_owned(), config.get("isin").unwrap().to_owned());
        match_engine.run(order_entry_rx, engine_msg_out_tx);
    });

    wait_50_millis();

    let engine_msg_in_thread = thread::spawn(|| {
        initialize_engine_msg_in_receiver(order_entry_tx).expect("failed to initialize engine MSG_IN thread");
    });

    wait_50_millis();

    let engine_msg_out_thread = thread::spawn(|| {
        initialize_engine_msg_out_submitter(engine_msg_out_rx).expect("failed to initialize engine MSG_OUT thread");
    });

    engine_thread.join().unwrap();
    engine_msg_out_thread.join().unwrap();
    engine_msg_in_thread.join().unwrap();

    Ok(())
}

fn initialize_engine_msg_in_receiver(order_entry_tx: Sender<Order>) -> Result<(), Box<dyn Error>> {
    use socket2::{Domain, Protocol, Socket, Type};
    let udp_multicast_socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).unwrap();
    udp_multicast_socket.set_reuse_address(true).expect("failed to set reuse address");
    udp_multicast_socket.set_reuse_port(true).expect("failed to set reuse port");
    udp_multicast_socket.bind(&SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, *ENGINE_MSG_IN_PORT).into()).unwrap();

    let udp_socket = std::net::UdpSocket::from(udp_multicast_socket);
    let send_addr = "0.0.0.0:3000".parse::<SocketAddr>().unwrap();
    let mut buffer = [0; 32768];

    println!("Initialized Engine MSG_IN multicast on port {}", *ENGINE_MSG_IN_PORT);

    loop {
        let (size, addr) = udp_socket.recv_from(&mut buffer).unwrap();
        let inbound_engine_message: Vec<InboundEngineMessage> = bitcode::decode(&buffer[..size]).unwrap();
        for msg_in in inbound_engine_message {
            match msg_in.message {
                EngineMessage::NewOrder(new) => {
                    order_entry_tx.send(Order::New(LimitOrder {
                        client_id: new.client_id,
                        id: random::<u32>(),
                        action: new.order_action,
                        px: new.px,
                        qty: new.qty,
                        placed_time: epoch_nanos(),
                    })).unwrap()
                }
                EngineMessage::CancelOrder(cancel) => {
                    order_entry_tx.send(Order::Cancel(CancelOrder {
                        client_id: cancel.client_id,
                        action: cancel.order_action,
                        id: cancel.order_id,
                    })).unwrap()
                }
                _ => {
                    unimplemented!()
                }
            }
        }
    }
}

fn initialize_engine_msg_out_submitter(rx: Receiver<OutboundEngineMessage>) -> Result<(), Box<dyn Error>> {
    use socket2::{Domain, Type};
    let udp_multicast_socket = socket2::Socket::new(Domain::IPV4, Type::DGRAM, Some(socket2::Protocol::UDP)).expect("failed to create UDP socket");
    udp_multicast_socket.set_reuse_address(true).expect("failed to set reuse address");
    udp_multicast_socket.set_reuse_port(true).expect("failed to set reuse port");

    let udp_socket = std::net::UdpSocket::from(udp_multicast_socket);
    let send_addr = "0.0.0.0:3500".parse::<SocketAddr>().unwrap();

    println!("Initialized Engine MSG_OUT multicast on port {}", *ENGINE_MSG_OUT_PORT);

    while let Ok(outbound_engine_message) = rx.recv() {
        let encoded: Vec<u8> = bitcode::encode(&outbound_engine_message);
        // println!("PUSHING {:?}", outbound_engine_message.outbound_message);
        udp_socket.send_to(&encoded, send_addr).expect("TODO: panic message");
    }
    Ok(())
}