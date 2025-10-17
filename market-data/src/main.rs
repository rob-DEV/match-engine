use common::domain::messaging::{EngineMessage, SequencedEngineMessage};
use lazy_static::lazy_static;
use std::net::{Ipv4Addr, SocketAddrV4};
use common::network::network_constants::MAX_UDP_PACKET_SIZE;
use common::network::udp_socket::multicast_udp_socket;

lazy_static! {
    pub static ref ENGINE_MSG_OUT_PORT: u16 = 3500;
}

fn main() {
    let udp_socket = multicast_udp_socket(*ENGINE_MSG_OUT_PORT, true);
    let mut buffer = [0; MAX_UDP_PACKET_SIZE];
    
    println!(
        "Initialized MSG_OUT -> Market Data Reporter multicast on port {}",
        *ENGINE_MSG_OUT_PORT
    );

    loop {
        match udp_socket.recv_from(&mut buffer) {
            Ok((size, _)) => {
                let outbound_engine_message: SequencedEngineMessage =
                    bitcode::decode(&buffer[..size]).unwrap();

                let outbound_message_type = &outbound_engine_message.message;

                match outbound_message_type {
                    EngineMessage::NewOrderAck(new_ack) => {
                        println!("{:?}", new_ack);
                    }
                    EngineMessage::CancelOrderAck(cancel_ack) => {
                        println!("{:?}", cancel_ack);
                    }
                    EngineMessage::TradeExecution(execution) => {
                        println!("{:?}", execution);
                    }
                    _ => {
                        unimplemented!()
                    }
                }
            }
            Err(_) => {}
        }
    }
}
