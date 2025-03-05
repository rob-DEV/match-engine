use crate::ENGINE_MSG_OUT_PORT;
use common::domain::messaging::SequencedEngineMessage;
use common::network::udp_socket::multicast_udp_socket;
use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc::Receiver;

pub fn initialize_engine_msg_out_thread(rx: Receiver<SequencedEngineMessage>) -> ! {
    println!("Initializing Engine MSG_OUT multicast on port {}", *ENGINE_MSG_OUT_PORT);
    let msg_out_socket = multicast_udp_socket(*ENGINE_MSG_OUT_PORT, false);
    engine_msg_out_to_multicast(&rx, &msg_out_socket)
}

pub fn engine_msg_out_to_multicast(mut rx: &Receiver<SequencedEngineMessage>, mut udp_socket: &UdpSocket) -> ! {
    let send_addr = "0.0.0.0:3500".parse::<SocketAddr>().unwrap();

    while let Ok(outbound_engine_message) = rx.recv() {
        let encoded: Vec<u8> = bitcode::encode(&outbound_engine_message);
        udp_socket.send_to(&encoded, send_addr).expect("TODO: panic message");
    }

    loop {}
}
