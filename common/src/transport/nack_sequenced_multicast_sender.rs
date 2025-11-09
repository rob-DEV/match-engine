use crate::memory::ring_slot::RingSlot;
use crate::network::mutlicast::multicast_receiver;
use crate::network::network_constants::MAX_UDP_PACKET_SIZE;
use crate::transport::sequenced_message::{
    EngineMessage, SequenceNumber, SequencedEngineMessage, SequencedMessageNack,
};
use crate::transport::transport_constants::MAX_MESSAGE_RETRANSMISSION_RING;
use crate::util::time::system_nanos;

use bitcode::Buffer;
use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;

pub struct NackSequencedMulticastSender {
    socket: Arc<UdpSocket>,
    socket_addr: SocketAddr,
    sequence_number: SequenceNumber,
    encode_buf: Buffer,
    resend_ring: Arc<Vec<RingSlot<SequencedEngineMessage>>>,
}

impl NackSequencedMulticastSender {
    pub fn new(socket: UdpSocket, socket_addr: SocketAddr) -> Self {
        let resend_ring: Arc<Vec<RingSlot<SequencedEngineMessage>>> = Arc::new(
            (0..MAX_MESSAGE_RETRANSMISSION_RING)
                .map(|_| RingSlot::new())
                .collect(),
        );

        let send_socket = Arc::new(socket);

        // Spawn NACK listener/retransmitter thread
        {
            let nack_ring = resend_ring.clone();
            let nack_socket = multicast_receiver(9000);

            std::thread::spawn(move || {
                let mut rx_buf = [0u8; MAX_UDP_PACKET_SIZE];
                let mut encode_buf = Buffer::new();

                loop {
                    let Ok((size, remote)) = nack_socket.recv_from(&mut rx_buf) else {
                        continue;
                    };

                    let nack: SequencedMessageNack = match bitcode::decode(&rx_buf[..size]) {
                        Ok(n) => n,
                        Err(_) => continue,
                    };

                    // println!("Got nack for {}", nack.requested_sequence_number);

                    let req = nack.requested_sequence_number;
                    let idx = (req as usize) % MAX_MESSAGE_RETRANSMISSION_RING;
                    let slot = &nack_ring[idx];

                    if let Some(msg) = slot.load(req) {
                        let enc = encode_buf.encode(msg);
                        let _ = nack_socket.send_to(enc, remote);
                    }
                }
            });
        }

        Self {
            socket: send_socket,
            socket_addr,
            sequence_number: 1,
            encode_buf: Buffer::new(),
            resend_ring,
        }
    }

    pub fn send(&mut self, engine_message: EngineMessage) {
        let seq = self.sequence_number;

        let msg = SequencedEngineMessage {
            sequence_number: seq,
            message: engine_message,
            sent_time: system_nanos(),
        };

        // Store message in ring
        let idx = (seq as usize) % MAX_MESSAGE_RETRANSMISSION_RING;
        let slot = &self.resend_ring[idx];
        slot.store(seq, msg);

        // Encode and send
        let enc = self.encode_buf.encode(slot.load(seq).unwrap());
        let _ = self.socket.send_to(enc, self.socket_addr);

        self.sequence_number = seq + 1;
    }
}
