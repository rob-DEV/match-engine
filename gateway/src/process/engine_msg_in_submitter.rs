use crate::message::GatewayMessage;
use crate::ENGINE_MSG_IN_PORT;
use bitcode::Buffer;
use common::transport::sequenced_message::{EngineMessage, SequencedEngineMessage};
use common::network::mutlicast::multicast_sender;
use common::util::time::system_nanos;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::mpsc::Receiver;

pub fn initialize_engine_msg_in_message_submitter(
    engine_msg_in_port: u16,
    rx: Receiver<GatewayMessage>,
) -> Result<(), Box<dyn Error>> {
    let udp_socket = multicast_sender();
    let send_addr = "239.255.0.1:3000".parse::<SocketAddr>().unwrap();

    println!(
        "Initialized Gateway -> MSG_IN multicast on port {}",
        *ENGINE_MSG_IN_PORT
    );

    let mut sequence = 1;
    let mut msg_in_encoding_buffer = Buffer::new();

    loop {
        while let Ok(inbound_engine_message) = rx.try_recv() {
            let message_in = match inbound_engine_message {
                GatewayMessage::LimitOrder(new) => SequencedEngineMessage {
                    sequence_number: sequence,
                    message: EngineMessage::NewOrder(new),
                    sent_time: system_nanos(),
                },
                GatewayMessage::MarketOrder(_) => {
                    unimplemented!()
                }
                GatewayMessage::CancelOrder(cancel) => SequencedEngineMessage {
                    sequence_number: sequence,
                    message: EngineMessage::CancelOrder(cancel),
                    sent_time: system_nanos(),
                },
            };

            let encoded: &[u8] = msg_in_encoding_buffer.encode(&message_in);
            udp_socket.send_to(&encoded, &send_addr).unwrap();

            let mut ack_bits = [0u8; 4];
            udp_socket
                .recv_from(&mut ack_bits)
                .expect("TODO: panic message");

            let id: u32 = u32::from_le_bytes([ack_bits[0], ack_bits[1], ack_bits[2], ack_bits[3]]);
            assert_eq!(id, sequence);

            sequence += 1;
        }
    }
}
