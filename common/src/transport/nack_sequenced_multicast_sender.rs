use crate::memory::ring_slot::RingSlot;
use crate::network::network_constants::MAX_UDP_PACKET_SIZE;
use crate::transport::sequenced_message::{
    EngineMessage, SequenceNumber, SequencedEngineMessage, SequencedMessageNack,
};
use crate::util::time::system_nanos;
use bitcode::Buffer;
use std::net::{SocketAddr, UdpSocket};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread::JoinHandle;

const MAX_MESSAGE_RETRANSMISSION_CACHE: usize = 2048;

pub struct NackSequencedMulticastSender {
    socket: Arc<UdpSocket>,
    socket_addr: SocketAddr,

    sequence_number: SequenceNumber,
    sequenced_message_encoding_buffer: Buffer,

    resend_ring: Arc<Vec<RingSlot<SequencedEngineMessage>>>,
    nack_handler: JoinHandle<()>,
}

impl NackSequencedMulticastSender {
    pub fn new(socket: Box<UdpSocket>, socket_addr: SocketAddr) -> Self {
        let ring: Arc<Vec<RingSlot<SequencedEngineMessage>>> =
            Arc::new((0..MAX_UDP_PACKET_SIZE).map(|_| RingSlot::new()).collect());
        let socket: Arc<UdpSocket> = Arc::from(socket);

        let nack_ring = ring.clone();
        let nack_socket = socket.clone();

        let nack = std::thread::spawn(move || {
            let mut udp_datagram_buffer = [0; MAX_UDP_PACKET_SIZE];
            let mut sequenced_message_encoding_buffer = Buffer::new();
            loop {
                match nack_socket.recv_from(&mut udp_datagram_buffer) {
                    Ok((size, nack_sock_addr)) => {
                        let nack: SequencedMessageNack =
                            bitcode::decode(&udp_datagram_buffer[..size]).unwrap();

                        let requested_sequence = nack.requested_sequence_number;

                        let index = requested_sequence as usize % MAX_MESSAGE_RETRANSMISSION_CACHE;
                        let slot = &nack_ring[index];
                        let msg = unsafe { &*slot.msg.get() };

                        // TODO: fix the double+ encode
                        let encoded: &[u8] = sequenced_message_encoding_buffer.encode(msg);

                        nack_socket
                            .send_to(&encoded, nack_sock_addr)
                            .expect("TODO: panic message");
                    }
                    _ => {}
                }
            }
        });

        NackSequencedMulticastSender {
            sequence_number: 1,
            resend_ring: ring,
            socket,
            socket_addr,
            sequenced_message_encoding_buffer: Buffer::new(),
            nack_handler: nack,
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

        // Add to ring
        let index = self.sequence_number as usize % MAX_MESSAGE_RETRANSMISSION_CACHE;
        let slot = &self.resend_ring[index];

        unsafe {
            *slot.msg.get() = sequenced_engine_message;
        }

        slot.seq.store(self.sequence_number, Ordering::Release);

        self.sequence_number += 1;
    }
}
