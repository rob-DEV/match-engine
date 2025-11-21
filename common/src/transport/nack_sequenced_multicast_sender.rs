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

use crate::transport::zero_alloc::RawWireMessage;
use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;
use std::thread;
use crate::serialize::serialize::{as_bytes, from_bytes};

const MAX_FLUSH_GAP_NS: u64 = 10_000;

pub struct NackSequencedMulticastSender {
    socket: Arc<UdpSocket>,
    socket_addr: SocketAddr,
    sequence_number: SequenceNumber,
    resend_ring: Arc<Vec<TransportRingSlot<SequencedEngineMessage>>>,
    raw_batch: RawWireMessage,
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

            loop {
                match nack_socket.recv_from(&mut rx_buf) {
                    Ok((size, remote)) => {
                        let nack: &SequencedMessageRangeNack =
                            from_bytes::<SequencedMessageRangeNack>(&rx_buf[..size]);

                        for seq in nack.start..=nack.end {
                            let idx = (seq as usize) % MAX_MESSAGE_RETRANSMISSION_RING;
                            let slot = &nack_ring[idx];

                            if let Some(msg) = slot.load(seq) {
                                // TODO: batch up nacks as well
                                let mut raw_wire_msg: RawWireMessage = RawWireMessage::default();
                                raw_wire_msg.batch_size = 1;
                                raw_wire_msg.batch[0] = msg;
                                let _ = nack_socket.send_to(as_bytes(&raw_wire_msg), remote);
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
            resend_ring,
            raw_batch: RawWireMessage::default(),
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

        self.raw_batch.batch[self.raw_batch.batch_size as usize] = slot.load(seq).unwrap();
        self.raw_batch.batch_size += 1;

        let now = system_nanos();
        if self.raw_batch.batch_size == MAX_UDP_MSG_BATCH_SIZE as u16
            || now - self.last_flush_ns > MAX_FLUSH_GAP_NS
        {
            self.socket
                .send_to(as_bytes(&self.raw_batch), self.socket_addr)
                .unwrap();

            self.raw_batch.batch_size = 0;
            self.last_flush_ns = now;
        }

        self.sequence_number = seq + 1;
    }
}
