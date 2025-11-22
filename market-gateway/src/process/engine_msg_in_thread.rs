use common::network::mutlicast::multicast_sender;
use common::transport::nack_sequenced_multicast_sender::NackSequencedMulticastSender;
use common::transport::sequenced_message::EngineMessage;
use core_affinity::CoreId;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::thread;
use std::thread::JoinHandle;
use tokio::sync::mpsc::Receiver;

pub fn msg_in_thread(
    msg_in_port: u16,
    pinned_msg_in_core: CoreId,
    mut rx_oe_queue: Receiver<EngineMessage>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        // core_affinity::set_for_current(pinned_msg_in_core);
        let msg_in_socket = multicast_sender();
        let send_addr = SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::new(239, 255, 0, 1),
            msg_in_port,
        ));

        let mut multicast_sender =
            NackSequencedMulticastSender::new(msg_in_socket, send_addr, 9000);

        loop {
            while let Ok(inbound_engine_message) = rx_oe_queue.try_recv() {
                multicast_sender.send(inbound_engine_message);
            }
        }
    })
}
