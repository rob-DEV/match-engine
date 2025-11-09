use crate::network::network_constants::MAX_UDP_PACKET_SIZE;
use crate::transport::sequenced_message::{
    SequenceNumber, SequencedEngineMessage, SequencedMessageNack,
};
use bitcode::Buffer;
use std::net::UdpSocket;
use std::option::Option;

const MAX_MESSAGE_RETRANSMISSION_CACHE: usize = 2048;

pub struct NackSequencedMulticastReceiver {
    socket: Box<UdpSocket>,
    received_msg_buffer: Vec<SequencedEngineMessage>,
    last_seen_sequence_number: SequenceNumber,

    ack_encoding_buffer: Buffer,
    udp_datagram_buffer: [u8; MAX_UDP_PACKET_SIZE],
}

impl NackSequencedMulticastReceiver {
    pub fn new(socket: Box<UdpSocket>) -> Self {
        NackSequencedMulticastReceiver {
            socket,
            received_msg_buffer: Vec::with_capacity(MAX_MESSAGE_RETRANSMISSION_CACHE),
            last_seen_sequence_number: 0,
            ack_encoding_buffer: Buffer::new(),
            udp_datagram_buffer: [0u8; MAX_UDP_PACKET_SIZE],
        }
    }

    pub fn try_recv(&mut self) -> Option<SequencedEngineMessage> {
        match self.socket.recv_from(&mut self.udp_datagram_buffer) {
            Ok((size, sock_addr)) => {
                let inbound_sequence_message: SequencedEngineMessage =
                    bitcode::decode(&self.udp_datagram_buffer[..size]).unwrap();

                if inbound_sequence_message.sequence_number != self.last_seen_sequence_number + 1 {
                    // missed a msg send nack and continue
                    let nack = SequencedMessageNack {
                        requested_sequence_number: self.last_seen_sequence_number + 1,
                    };
                    let encoded_nack: &[u8] = self.ack_encoding_buffer.encode(&nack);

                    self.socket
                        .send_to(&encoded_nack, sock_addr)
                        .expect("TODO: panic message");

                    return None;
                }

                assert_eq!(
                    inbound_sequence_message.sequence_number,
                    self.last_seen_sequence_number + 1
                );
                self.last_seen_sequence_number = inbound_sequence_message.sequence_number;

                return Some(inbound_sequence_message);
            }
            Err(_) => {
                return None;
            }
        }
    }
}
