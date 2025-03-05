use common::domain::{OutboundEngineMessage, OutboundMessage};
use lazy_static::lazy_static;
use std::net::{Ipv4Addr, SocketAddrV4};

lazy_static! {
    pub static ref ENGINE_MSG_OUT_PORT: u16 = 3500;
}

fn main() {
    use socket2::{Domain, Type};
    let udp_multicast_socket = socket2::Socket::new(Domain::IPV4, Type::DGRAM, Some(socket2::Protocol::UDP)).expect("failed to create UDP socket");
    udp_multicast_socket.set_reuse_address(true).expect("failed to set reuse address");
    udp_multicast_socket.set_reuse_port(true).expect("failed to set reuse port");
    udp_multicast_socket.bind(&SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, *ENGINE_MSG_OUT_PORT).into()).expect("failed to bind UDP socket");

    let udp_socket = std::net::UdpSocket::from(udp_multicast_socket);

    let mut buffer = [0; 128];

    println!("Initialized MSG_OUT -> Gateway multicast on port {}", *ENGINE_MSG_OUT_PORT);

    loop {
        match udp_socket.recv_from(&mut buffer) {
            Ok((size, _)) => {
                let outbound_engine_message: OutboundEngineMessage = bitcode::decode(&buffer[..size]).unwrap();
                let outbound_message_type = &outbound_engine_message.outbound_message;

                match outbound_message_type {
                    OutboundMessage::NewOrderAck(new_order_ack) => {
                        println!("Received NewOrderAck message");
                    }
                    OutboundMessage::CancelOrderAck(cancel_order_ack) => {
                        println!("Received CancelOrderAck message");
                    }
                    OutboundMessage::TradeExecution(execution) => {
                        println!("Received TradeExecution message");
                    }
                    _ => { unimplemented!() }
                }
            }
            Err(_) => {}
        }
    }
}
