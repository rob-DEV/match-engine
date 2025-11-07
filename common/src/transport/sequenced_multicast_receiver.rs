use crate::network::network_constants::MAX_UDP_PACKET_SIZE;
use crate::transport::sequenced_message::{
    SequenceNumber, SequencedEngineMessage, SequencedMessageAck, Subscriber,
};
use bitcode::Buffer;
use std::net::UdpSocket;
use std::option::Option;

pub struct SequencedMulticastReceiver {
    subscriber: Subscriber,
    last_seen_sequence_number: SequenceNumber,
    socket: Box<UdpSocket>,

    ack_encoding_buffer: Buffer,
    udp_datagram_buffer: [u8; MAX_UDP_PACKET_SIZE],
}

impl SequencedMulticastReceiver {
    pub fn new(socket: Box<UdpSocket>, subscriber: Subscriber) -> Self {
        SequencedMulticastReceiver {
            subscriber,
            last_seen_sequence_number: 0,
            socket,
            ack_encoding_buffer: Buffer::new(),
            udp_datagram_buffer: [0u8; MAX_UDP_PACKET_SIZE],
        }
    }

    pub fn try_recv(&mut self) -> Option<SequencedEngineMessage> {
        match self.socket.recv_from(&mut self.udp_datagram_buffer) {
            Ok((size, sock_addr)) => {
                let inbound_sequence_message: SequencedEngineMessage =
                    bitcode::decode(&self.udp_datagram_buffer[..size]).unwrap();

                let ack = SequencedMessageAck {
                    subscriber: self.subscriber,
                    sequence_number: inbound_sequence_message.sequence_number,
                };

                let encoded_ack: &[u8] = self.ack_encoding_buffer.encode(&ack);

                self.socket
                    .send_to(&encoded_ack, sock_addr)
                    .expect("TODO: panic message");

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
