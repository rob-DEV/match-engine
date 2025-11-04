use common::network::mutlicast::multicast_receiver;
use common::network::network_constants::MAX_UDP_PACKET_SIZE;
use lazy_static::lazy_static;
use common::domain::messaging::{EngineMessage, SequencedEngineMessage};

lazy_static! {
    pub static ref ENGINE_MSG_OUT_PORT: u16 = 3500;
}

fn main() {
    let udp_socket = multicast_receiver(*ENGINE_MSG_OUT_PORT);
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
                        println!(
                            "{} -> {:?}",
                            outbound_engine_message.sequence_number, new_ack
                        );
                    }
                    EngineMessage::CancelOrderAck(cancel_ack) => {
                        println!(
                            "{} -> {:?}",
                            outbound_engine_message.sequence_number, cancel_ack
                        );
                    }
                    EngineMessage::TradeExecution(execution) => {
                        println!(
                            "{} -> {:?}",
                            outbound_engine_message.sequence_number, execution
                        );
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
