use crate::memory::ring_slot::TransportRingSlot;
use crate::network::mutlicast::multicast_sender;
use crate::network::network_constants::MAX_UDP_PACKET_SIZE;
use crate::transport::sequenced_message::{
    SequenceNumber, SequencedEngineMessage, SequencedMessageNack,
};
use crate::transport::transport_constants::MAX_MESSAGE_RETRANSMISSION_RING;
use crate::util::time::system_nanos;
use bitcode::Buffer;
use std::io::ErrorKind;
use std::net::{SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const NACK_INTERVAL_NS: u64 = 50_000;
pub struct NackSequencedMulticastReceiver {
    last_seen_sequence_number: SequenceNumber, // keep for try_recv() convenience
    last_seen_atomic: Arc<AtomicU32>,          // shared with the NACK thread
    transport_ring: Arc<Vec<TransportRingSlot<SequencedEngineMessage>>>,
}

impl NackSequencedMulticastReceiver {
    pub fn new(recv_socket: Box<UdpSocket>) -> Self {
        let recv_side_ring: Arc<Vec<TransportRingSlot<SequencedEngineMessage>>> = Arc::new(
            (0..MAX_MESSAGE_RETRANSMISSION_RING)
                .map(|_| TransportRingSlot::new())
                .collect(),
        );

        let last_seen_atomic = Arc::new(AtomicU32::new(0));

        // Single socket for NACK send and retrans receive (replies come back to the source port)
        let nack_io_socket = multicast_sender();
        let retrans_recv_socket = nack_io_socket.try_clone().unwrap();

        let nack_send_socket = nack_io_socket;

        let ring_for_main = recv_side_ring.clone();
        let ring_for_retrans = recv_side_ring.clone();
        let ring_for_nack_recv = recv_side_ring.clone();

        let last_seen_for_nack = last_seen_atomic.clone();

        // Thread: multicast feed
        thread::spawn(move || {
            let mut buf = [0u8; MAX_UDP_PACKET_SIZE];
            loop {
                match recv_socket.recv_from(&mut buf) {
                    Ok((size, _src)) => {
                        if let Ok(msg) = bitcode::decode::<SequencedEngineMessage>(&buf[..size]) {
                            let idx =
                                msg.sequence_number as usize % MAX_MESSAGE_RETRANSMISSION_RING;
                            let slot = &ring_for_main[idx];
                            slot.store(msg.sequence_number, msg);
                        }
                        continue;
                    }
                    Err(e) if e.kind() == ErrorKind::WouldBlock => thread::yield_now(),
                    Err(e) => eprintln!("MAIN_RECV error: {:?}", e),
                }
            }
        });

        // Thread: retransmit receiver
        thread::spawn(move || {
            let mut buf = [0u8; MAX_UDP_PACKET_SIZE];

            loop {
                match retrans_recv_socket.recv_from(&mut buf) {
                    Ok((size, _src)) => {
                        if let Ok(msg) = bitcode::decode::<SequencedEngineMessage>(&buf[..size]) {
                            let seq = msg.sequence_number;
                            let idx = seq as usize % MAX_MESSAGE_RETRANSMISSION_RING;
                            let slot = &ring_for_retrans[idx];
                            slot.store(seq, msg);
                        }
                        continue;
                    }
                    Err(e) if e.kind() == ErrorKind::WouldBlock => thread::yield_now(),
                    Err(e) => eprintln!("RETRANS_RECV error: {:?}", e),
                }
            }
        });

        // Thread: periodic NACK sender — uses the shared last_seen_atomic and outstanding map.
        thread::spawn(move || {
            let mut encoding_buf = Buffer::new();

            loop {
                let expected = last_seen_for_nack.load(Ordering::Acquire) + 1;
                // check ring to see if present
                let idx = (expected as usize) % MAX_MESSAGE_RETRANSMISSION_RING;
                let slot = &ring_for_nack_recv[idx];

                let now = system_nanos();
                let should_send = now.saturating_sub(slot.last_nack_ns.load(Ordering::Acquire))
                    >= NACK_INTERVAL_NS;
                if should_send {
                    let nack = SequencedMessageNack {
                        requested_sequence_number: expected,
                    };
                    let encoded: &[u8] = encoding_buf.encode(&nack);

                    let nack_listen_addr = "239.255.0.1:9000".parse::<SocketAddr>().unwrap();
                    let _ = nack_send_socket.send_to(&encoded, nack_listen_addr);

                    slot.last_nack_ns.store(now, Ordering::Release);
                }

                thread::sleep(Duration::from_micros(5));
            }
        });

        NackSequencedMulticastReceiver {
            last_seen_sequence_number: 0,
            last_seen_atomic,
            transport_ring: recv_side_ring,
        }
    }

    pub fn try_recv(&mut self) -> Option<&SequencedEngineMessage> {
        let expected_sequence_number = self.last_seen_sequence_number + 1;
        let index = expected_sequence_number as usize % MAX_MESSAGE_RETRANSMISSION_RING;
        let slot = &self.transport_ring[index];

        if let Some(msg) = slot.load(expected_sequence_number) {
            // println!("Found!: index {} seq: {}", index, msg.sequence_number);
            self.last_seen_sequence_number = msg.sequence_number;
            // Update atomic
            self.last_seen_atomic
                .store(self.last_seen_sequence_number, Ordering::Release);

            // Remove sequence number from NACK map
            slot.set_nack(0);
            return Some(msg);
        } else {
            // Register a NACK and handle in timer thread
            let now = system_nanos();
            let last = slot.last_nack();
            if last == 0 {
                slot.set_nack(now);
            }
            return None;
        }
    }
}
