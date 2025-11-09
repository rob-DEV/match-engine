use crate::network::network_constants::MAX_UDP_PACKET_SIZE;
use crate::transport::sequenced_message::{
    EngineMessage, SequenceNumber, SequencedEngineMessage, SequencedMessageAck, Subscriber,
};
use crate::util::time::system_nanos;
use bitcode::Buffer;
use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};

pub struct AckSequencedMulticastSender {
    sequence_number: SequenceNumber,
    sequenced_subscribers: HashMap<Subscriber, SequenceNumber>,
    socket: Box<UdpSocket>,
    socket_addr: SocketAddr,

    sequenced_message_encoding_buffer: Buffer,
    udp_datagram_buffer: [u8; MAX_UDP_PACKET_SIZE],
}

impl AckSequencedMulticastSender {
    pub fn new(
        socket: Box<UdpSocket>,
        socket_addr: SocketAddr,
        subscribers: Vec<Subscriber>,
    ) -> Self {
        AckSequencedMulticastSender {
            sequence_number: 1,
            sequenced_subscribers: subscribers.iter().map(|sub| (*sub, 0)).collect(),
            socket,
            socket_addr,
            sequenced_message_encoding_buffer: Buffer::new(),
            udp_datagram_buffer: [0u8; MAX_UDP_PACKET_SIZE],
        }
    }

    pub fn send(&mut self, engine_message: EngineMessage) {
        let sequenced_engine_message = SequencedEngineMessage {
            sequence_number: self.sequence_number,
            message: engine_message,
            sent_time: system_nanos(),
        };

        // Send
        let encoded: &[u8] = self
            .sequenced_message_encoding_buffer
            .encode(&sequenced_engine_message);

        self.socket
            .send_to(&encoded, self.socket_addr)
            .expect("TODO: panic message");

        self.wait_for_subscriber_acks();

        self.sequence_number += 1;
    }

    fn wait_for_subscriber_acks(&mut self) {
        for _ in 0..self.sequenced_subscribers.len() {
            match self.socket.recv_from(&mut self.udp_datagram_buffer) {
                Ok((size, _)) => {
                    let ack: SequencedMessageAck =
                        bitcode::decode(&self.udp_datagram_buffer[..size]).unwrap();

                    let subscriber = self.sequenced_subscribers.get_mut(&ack.subscriber).unwrap();

                    assert_eq!(ack.sequence_number, *subscriber + 1);
                    *subscriber += 1;
                }
                _ => {}
            }
        }
    }
}
