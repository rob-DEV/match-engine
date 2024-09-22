use crate::domain::order::{CancelOrder, LimitOrder, Order};
use common::engine::{InboundEngineMessage, InboundMessage, OutboundEngineMessage};
use lazy_static::lazy_static;
use rand::random;
use std::error::Error;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::{env, thread};

mod engine;
mod domain;
mod util;

lazy_static! {
    pub static ref ENGINE_MSG_IN_PORT: u16 = env::var("ENGINE_PORT").unwrap_or("3000".to_owned()).parse::<u16>().unwrap();
    pub static ref ENGINE_MSG_OUT_PORT: u16 = env::var("ENGINE_PORT").unwrap_or("3500".to_owned()).parse::<u16>().unwrap();
}


fn main() -> Result<(), Box<dyn Error>> {
    println!("--- Initializing Match Engine ---");

    let (engine_msg_out_tx, engine_msg_out_rx): (Sender<OutboundEngineMessage>, Receiver<OutboundEngineMessage>) = mpsc::channel();
    let (order_entry_tx, order_entry_rx): (Sender<Order>, Receiver<Order>) = mpsc::channel();

    let match_engine = engine::match_engine::MatchEngine::new();
    match_engine.run(order_entry_rx, engine_msg_out_tx);

    let engine_msg_in_thread = thread::spawn(|| {
        initialize_engine_msg_in_receiver(order_entry_tx).expect("failed to initialize engine MSG_IN thread");
    });

    let engine_msg_out_thread = thread::spawn(|| {
        initialize_engine_msg_out_submitter(engine_msg_out_rx).expect("failed to initialize engine MSG_OUT thread");
    });

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
    let mut buffer = [0; 1024];

    println!("Initialized Engine MSG_IN multicast on port {}", *ENGINE_MSG_IN_PORT);

    loop {
        let (size, addr) = udp_socket.recv_from(&mut buffer).unwrap();
        let inbound_engine_message: InboundEngineMessage = bitcode::decode(&buffer[..size]).unwrap();

        match inbound_engine_message.inbound_message {
            InboundMessage::NewOrder(new) => {
                order_entry_tx.send(Order::New(LimitOrder {
                    id: random::<u32>(),
                    action: new.order_action,
                    px: new.px,
                    qty: new.qty,
                    placed_time: 0,
                })).unwrap()
            }
            InboundMessage::CancelOrder(cancel) => {
                order_entry_tx.send(Order::Cancel(CancelOrder {
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
        udp_socket.send_to(&encoded, send_addr).expect("TODO: panic message");
    }
    Ok(())
}