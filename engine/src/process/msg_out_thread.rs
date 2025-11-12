use crate::ENGINE_MSG_OUT_PORT;
use common::network::mutlicast::multicast_sender;
use common::transport::nack_sequenced_multicast_sender::NackSequencedMulticastSender;
use common::transport::sequenced_message::EngineMessage;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::process::exit;
use std::sync::mpsc::Receiver;

pub fn initialize_engine_msg_out_thread(rx: Receiver<EngineMessage>) -> ! {
    println!(
        "Initializing Engine MSG_OUT multicast on port {}",
        *ENGINE_MSG_OUT_PORT
    );
    let msg_out_socket = multicast_sender();

    engine_msg_out_to_multicast(&rx, msg_out_socket);
}

pub fn engine_msg_out_to_multicast(rx: &Receiver<EngineMessage>, udp_socket: UdpSocket) -> ! {
    let send_addr = SocketAddr::V4(SocketAddrV4::new(
        Ipv4Addr::new(239, 255, 0, 1),
        *ENGINE_MSG_OUT_PORT,
    ));
    let mut multicast_sender = NackSequencedMulticastSender::new(udp_socket, send_addr, 9001);

    while let Ok(outbound_engine_message) = rx.recv() {
        multicast_sender.send(outbound_engine_message)
    }

    exit(-1);
}
