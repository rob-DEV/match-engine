use crate::memory::ring_slot::TransportRingSlot;
use crate::network::mutlicast::multicast_receiver;
use crate::network::network_constants::MAX_UDP_PACKET_SIZE;
use crate::transport::sequenced_message::{
    EngineMessage, SequenceNumber, SequencedEngineMessage, SequencedMessageRangeNack,
    MAX_UDP_MSG_BATCH_SIZE,
};
use crate::transport::transport_constants::MAX_MESSAGE_RETRANSMISSION_RING;
use crate::util::time::system_nanos;
use std::io::ErrorKind;

use bitcode::Buffer;
use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;
use std::thread;

const MAX_FLUSH_GAP_NS: u64 = 10_000;

pub struct NackSequencedMulticastSender {
    socket: Arc<UdpSocket>,
    socket_addr: SocketAddr,
    sequence_number: SequenceNumber,
    encode_buf: Buffer,
    resend_ring: Arc<Vec<TransportRingSlot<SequencedEngineMessage>>>,
    batch: Vec<SequencedEngineMessage>,
    batch_idx: usize,
    last_flush_ns: u64,
}

impl NackSequencedMulticastSender {
    pub fn new(socket: UdpSocket, socket_addr: SocketAddr, nack_port: u16) -> Self {
        let resend_ring: Arc<Vec<TransportRingSlot<SequencedEngineMessage>>> = Arc::new(
            (0..MAX_MESSAGE_RETRANSMISSION_RING)
                .map(|_| TransportRingSlot::new())
                .collect(),
        );

        let send_socket = Arc::new(socket);

        // Spawn NACK listener/retransmitter thread
        let nack_ring = resend_ring.clone();
        let nack_socket = multicast_receiver(nack_port);

        thread::spawn(move || {
            let mut rx_buf = [0u8; MAX_UDP_PACKET_SIZE];
            let mut encode_buf = Buffer::new();

            loop {
                match nack_socket.recv_from(&mut rx_buf) {
                    Ok((size, remote)) => {
                        let nack: SequencedMessageRangeNack = match bitcode::decode(&rx_buf[..size])
                        {
                            Ok(n) => n,
                            Err(_) => continue,
                        };

                        for seq in nack.start..=nack.end {
                            let idx = (seq as usize) % MAX_MESSAGE_RETRANSMISSION_RING;
                            let slot = &nack_ring[idx];

                            if let Some(msg) = slot.load(seq) {
                                let enc = encode_buf.encode(&msg);
                                let _ = nack_socket.send_to(enc, remote);
                            }
                        }
                    }
                    Err(e) if e.kind() == ErrorKind::WouldBlock => thread::yield_now(),
                    Err(e) => eprintln!("RETRANS_RECV error: {:?}", e),
                }
            }
        });

        Self {
            socket: send_socket,
            socket_addr,
            sequence_number: 1,
            encode_buf: Buffer::new(),
            resend_ring,
            batch: Vec::with_capacity(MAX_UDP_MSG_BATCH_SIZE),
            batch_idx: 0,
            last_flush_ns: 0,
        }
    }

    pub fn send(&mut self, engine_message: EngineMessage) {
        let seq = self.sequence_number;

        let msg = SequencedEngineMessage {
            sequence_number: seq,
            message: engine_message,
            sent_time: system_nanos(),
        };

        let idx = (seq as usize) % MAX_MESSAGE_RETRANSMISSION_RING;
        let slot = &self.resend_ring[idx];
        slot.store(seq, msg);

        // Encode and send
        self.batch.push(slot.load(seq).unwrap());
        self.batch_idx += 1;

        let now = system_nanos();
        if self.batch_idx == MAX_UDP_MSG_BATCH_SIZE || now - self.last_flush_ns > MAX_FLUSH_GAP_NS {
            let enc = self.encode_buf.encode(&self.batch);
            let _ = self.socket.send_to(enc, self.socket_addr);

            self.last_flush_ns = now;
            self.batch_idx = 0;
            self.batch.clear()
        }

        self.sequence_number = seq + 1;
    }
}
