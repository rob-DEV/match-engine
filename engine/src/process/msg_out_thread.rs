use crate::ENGINE_MSG_OUT_PORT;
use bitcode::Buffer;
use common::transport::sequenced_message::SequencedEngineMessage;
use common::network::mutlicast::multicast_sender;
use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc::Receiver;

pub fn initialize_engine_msg_out_thread(rx: Receiver<SequencedEngineMessage>) -> ! {
    println!(
        "Initializing Engine MSG_OUT multicast on port {}",
        *ENGINE_MSG_OUT_PORT
    );
    let msg_out_socket = multicast_sender();
    engine_msg_out_to_multicast(&rx, &msg_out_socket)
}

pub fn engine_msg_out_to_multicast(
    mut rx: &Receiver<SequencedEngineMessage>,
    mut udp_socket: &UdpSocket,
) -> ! {
    let send_addr = "239.255.0.1:3500".parse::<SocketAddr>().unwrap();
    let mut msg_out_encoding_buffer = Buffer::new();

    while let Ok(outbound_engine_message) = rx.recv() {
        let encoded: &[u8] = msg_out_encoding_buffer.encode(&outbound_engine_message);

        udp_socket
            .send_to(&encoded, send_addr)
            .expect("TODO: panic message");
    }

    loop {}
}
