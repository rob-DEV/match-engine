use crate::memory::ring_buffer::RingBuffer;
use crate::memory::ring_slot::TransportRingSlot;
use crate::network::mutlicast::multicast_sender;
use crate::network::network_constants::MAX_UDP_PACKET_SIZE;
use crate::transport::sequenced_message::{
    SequenceNumber, SequencedEngineMessage, SequencedMessageRangeNack,
};
use crate::transport::transport_constants::MAX_MESSAGE_RETRANSMISSION_RING;
use crate::util::time::system_nanos;
use bitcode::Buffer;
use std::io::ErrorKind;
use std::net::{SocketAddr, UdpSocket};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const NACK_INTERVAL_NS: u64 = 50_000;

pub struct NackSequencedMulticastReceiver {
    last_seen_sequence_number: SequenceNumber,
    transport_ring: Arc<Vec<TransportRingSlot<SequencedEngineMessage>>>,
    nack_ring: Arc<RingBuffer>,
}

impl NackSequencedMulticastReceiver {
    pub fn new(recv_socket: UdpSocket, nack_port: u16) -> Self {
        let recv_side_ring: Arc<Vec<TransportRingSlot<SequencedEngineMessage>>> = Arc::new(
            (0..MAX_MESSAGE_RETRANSMISSION_RING)
                .map(|_| TransportRingSlot::new())
                .collect(),
        );

        let nack_ring = Arc::new(RingBuffer::new(128));
        let nack_ring_for_nack_sender = nack_ring.clone();

        // Single socket for NACK send and retrans receive (replies come back to the source port)
        let nack_io_socket = multicast_sender();
        let retrans_recv_socket = nack_io_socket.try_clone().unwrap();

        let nack_send_socket = nack_io_socket;

        let ring_for_main = recv_side_ring.clone();
        let ring_for_retrans = recv_side_ring.clone();

        let mut ave_latency_count = 0;
        let mut latency: u64 = 0;

        // Thread: multicast feed
        thread::spawn(move || {
            let mut rx_buf = [0u8; MAX_UDP_PACKET_SIZE];
            loop {
                match recv_socket.recv_from(&mut rx_buf) {
                    Ok((size, _src)) => {
                        if let Ok(msgs) =
                            bitcode::decode::<Vec<SequencedEngineMessage>>(&rx_buf[..size])
                        {
                            for msg in msgs {
                                let idx =
                                    msg.sequence_number as usize % MAX_MESSAGE_RETRANSMISSION_RING;
                                let slot = &ring_for_main[idx];
                                slot.store(msg.sequence_number, msg);
                            }
                        }
                    }
                    Err(e) if e.kind() == ErrorKind::WouldBlock => thread::yield_now(),
                    Err(e) => eprintln!("MAIN_RECV error: {:?}", e),
                }
            }
        });

        // Thread: retransmit receiver
        thread::spawn(move || {
            let mut rx_buf = [0u8; MAX_UDP_PACKET_SIZE];

            loop {
                match retrans_recv_socket.recv_from(&mut rx_buf) {
                    Ok((size, _src)) => {
                        if let Ok(msg) = bitcode::decode::<SequencedEngineMessage>(&rx_buf[..size])
                        {
                            let seq = msg.sequence_number;
                            let idx = seq as usize % MAX_MESSAGE_RETRANSMISSION_RING;
                            let slot = &ring_for_retrans[idx];
                            slot.store(seq, msg);
                        }
                    }
                    Err(e) if e.kind() == ErrorKind::WouldBlock => thread::yield_now(),
                    Err(e) => eprintln!("RETRANS_RECV error: {:?}", e),
                }
            }
        });

        // Thread: periodic NACK sender — uses the shared last_seen_atomic and outstanding map.
        thread::spawn(move || {
            let mut encoding_buf = Buffer::new();
            let mut batch = Vec::<u32>::with_capacity(128);
            let nack_listen_addr = SocketAddr::from(([239, 255, 0, 1], nack_port));

            loop {
                // Drain the lock-free queue
                while let Some(seq) = nack_ring_for_nack_sender.pop() {
                    batch.push(seq);
                    if batch.len() >= 64 {
                        break;
                    }
                }

                if !batch.is_empty() {
                    // sort to detect contiguous ranges
                    batch.sort_unstable();
                    let mut start = batch[0];
                    let mut prev = start;

                    for &s in &batch[1..] {
                        if s != prev + 1 {
                            let nack = SequencedMessageRangeNack { start, end: prev };
                            let encoded: &[u8] = encoding_buf.encode(&nack);

                            let _ = nack_send_socket.send_to(&encoded, nack_listen_addr);
                            start = s;
                        }
                        prev = s;
                    }

                    // final range
                    let nack = SequencedMessageRangeNack { start, end: prev };
                    let encoded: &[u8] = encoding_buf.encode(&nack);

                    let _ = nack_send_socket.send_to(&encoded, nack_listen_addr);

                    batch.clear();
                }

                thread::sleep(Duration::from_micros(10000));
            }
        });

        NackSequencedMulticastReceiver {
            last_seen_sequence_number: 0,
            transport_ring: recv_side_ring,
            nack_ring,
        }
    }

    pub fn try_recv(&mut self) -> Option<SequencedEngineMessage> {
        let expected_sequence_number = self.last_seen_sequence_number + 1;
        let index = expected_sequence_number as usize % MAX_MESSAGE_RETRANSMISSION_RING;
        let slot = &self.transport_ring[index];

        if let Some(msg) = slot.load(expected_sequence_number) {
            self.last_seen_sequence_number = msg.sequence_number;
            slot.pending_nack.store(false, Ordering::Release);
            slot.last_nack_ns.store(0, Ordering::Relaxed);

            return Some(msg);
        } else {
            // Register a NACK and handle in timer thread
            // Only push to NACK ring if throttle allows
            let now = system_nanos();

            if slot.pending_nack.load(Ordering::Acquire) {
                let last = slot.last_nack_ns.load(Ordering::Relaxed);

                if now.wrapping_sub(last) >= NACK_INTERVAL_NS {
                    slot.last_nack_ns.store(now, Ordering::Relaxed);
                    self.nack_ring.push(expected_sequence_number);
                }

                return None;
            }

            slot.pending_nack.store(true, Ordering::Release);
            slot.last_nack_ns.store(now, Ordering::Relaxed);
            self.nack_ring.push(expected_sequence_number);

            return None;
        }
    }
}
