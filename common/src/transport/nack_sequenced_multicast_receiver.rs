use crate::memory::ring_slot::RingSlot;
use crate::network::mutlicast::multicast_sender;
use crate::network::network_constants::MAX_UDP_PACKET_SIZE;
use crate::transport::sequenced_message::{
    SequenceNumber, SequencedEngineMessage, SequencedMessageNack,
};
use crate::transport::transport_constants::MAX_MESSAGE_RETRANSMISSION_RING;
use crate::util::time::system_nanos;
use bitcode::Buffer;
use dashmap::DashMap;
use std::io::ErrorKind;
use std::net::{SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const NACK_INTERVAL_NS: u64 = 2_000_000;
pub struct NackSequencedMulticastReceiver {
    last_seen_sequence_number: SequenceNumber, // keep for try_recv() convenience
    last_seen_atomic: Arc<AtomicU32>,          // shared with the NACK thread
    msg_ring: Arc<Vec<RingSlot<SequencedEngineMessage>>>,
    outstanding_nacks: Arc<DashMap<SequenceNumber, u64>>, // shared map
}

impl NackSequencedMulticastReceiver {
    pub fn new(recv_socket: Box<UdpSocket>) -> Self {
        let recv_side_ring: Arc<Vec<RingSlot<SequencedEngineMessage>>> = Arc::new(
            (0..MAX_MESSAGE_RETRANSMISSION_RING)
                .map(|_| RingSlot::new())
                .collect(),
        );

        let last_seen_atomic = Arc::new(AtomicU32::new(0));
        let outstanding_nacks = Arc::new(DashMap::<SequenceNumber, u64>::new());

        // Socket for nack and retransmission
        let retrans_recv_socket = multicast_sender(); // replace with real retrans receiver in production
        let nack_send_socket = multicast_sender(); // replace with real nack sender (unicast) in production

        let ring_for_main = recv_side_ring.clone();
        let ring_for_retrans = recv_side_ring.clone();

        let outstanding_for_retrans = outstanding_nacks.clone();
        let outstanding_for_nack = outstanding_nacks.clone();
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
            let map = outstanding_for_retrans;

            loop {
                match retrans_recv_socket.recv_from(&mut buf) {
                    Ok((size, _src)) => {
                        if let Ok(msg) = bitcode::decode::<SequencedEngineMessage>(&buf[..size]) {
                            let seq = msg.sequence_number;
                            let idx = seq as usize % MAX_MESSAGE_RETRANSMISSION_RING;
                            let slot = &ring_for_retrans[idx];
                            slot.store(seq, msg);
                            // clear outstanding nack for this seq
                            map.remove(&seq);
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
            let mut map = outstanding_for_nack;

            loop {
                let expected = last_seen_for_nack.load(Ordering::Acquire) + 1;
                // check ring to see if present
                let idx = (expected as usize) % MAX_MESSAGE_RETRANSMISSION_RING;

                let now = system_nanos();
                let should_send = match map.get(&expected) {
                    None => true,
                    Some(last) => now.saturating_sub(*last) >= NACK_INTERVAL_NS,
                };
                if should_send {
                    let nack = SequencedMessageNack {
                        requested_sequence_number: expected,
                    };
                    let encoded: &[u8] = encoding_buf.encode(&nack);

                    let nack_listen_addr = "239.255.0.1:9000".parse::<SocketAddr>().unwrap();
                    let _ = nack_send_socket.send_to(&encoded, nack_listen_addr);
                    map.insert(expected, now);
                }

                thread::sleep(Duration::from_micros(5));
            }
        });

        NackSequencedMulticastReceiver {
            last_seen_sequence_number: 0,
            last_seen_atomic,
            msg_ring: recv_side_ring,
            outstanding_nacks,
        }
    }

    pub fn try_recv(&mut self) -> Option<&SequencedEngineMessage> {
        let expected_sequence_number = self.last_seen_sequence_number + 1;
        let index = expected_sequence_number as usize % MAX_MESSAGE_RETRANSMISSION_RING;
        let slot = &self.msg_ring[index];

        if let Some(msg) = slot.load(expected_sequence_number) {
            println!("Found!: index {} seq: {}", index, msg.sequence_number);
            self.last_seen_sequence_number = msg.sequence_number;
            // Update atomic
            self.last_seen_atomic
                .store(self.last_seen_sequence_number, Ordering::Release);

            // Remove sequence number from NACK map
            let mut map = &self.outstanding_nacks;
            map.remove(&expected_sequence_number);
            return Some(msg);
        } else {
            // Register a NACK and handle in timer thread
            let now = system_nanos();
            let mut map = &self.outstanding_nacks;
            map.entry(expected_sequence_number).or_insert(now);
            return None;
        }
    }
}
