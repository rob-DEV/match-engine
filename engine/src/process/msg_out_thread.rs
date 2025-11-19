use common::network::mutlicast::multicast_sender;
use common::transport::nack_sequenced_multicast_sender::NackSequencedMulticastSender;
use common::transport::sequenced_message::EngineMessage;
use core_affinity::CoreId;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::mpsc::Receiver;
use std::thread;
use std::thread::JoinHandle;

pub fn msg_out_thread(
    msg_out_port: u16,
    engine_msg_out_rx: Receiver<EngineMessage>,
    pinned_msg_out_core: CoreId,
) -> JoinHandle<()> {
    thread::spawn(move || {
        core_affinity::set_for_current(pinned_msg_out_core);
        let msg_out_socket = multicast_sender();

        let send_addr = SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::new(239, 255, 0, 1),
            msg_out_port,
        ));

        let mut multicast_sender =
            NackSequencedMulticastSender::new(msg_out_socket, send_addr, 9001);

        while let Ok(outbound_engine_message) = engine_msg_out_rx.recv() {
            multicast_sender.send(outbound_engine_message)
        }
    })
}
