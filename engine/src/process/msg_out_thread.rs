use crate::ENGINE_MSG_OUT_PORT;
use common::network::mutlicast::multicast_sender;
use common::transport::sequenced_message::EngineMessage;
use common::transport::ack_sequenced_multicast_sender::AckSequencedMulticastSender;
use common::transport::transport_constants::{MARKET_DATA_CHANNEL, MSG_OUT_CHANNEL};
use std::net::{SocketAddr, UdpSocket};
use std::process::exit;
use std::sync::mpsc::Receiver;

pub fn initialize_engine_msg_out_thread(rx: Receiver<EngineMessage>) -> ! {
    println!(
        "Initializing Engine MSG_OUT multicast on port {}",
        *ENGINE_MSG_OUT_PORT
    );
    let msg_out_socket = multicast_sender();

    engine_msg_out_to_multicast(&rx, Box::new(msg_out_socket));
}

pub fn engine_msg_out_to_multicast(rx: &Receiver<EngineMessage>, udp_socket: Box<UdpSocket>) -> ! {
    let send_addr = "239.255.0.1:3500".parse::<SocketAddr>().unwrap();
    let mut multicast_sender = AckSequencedMulticastSender::new(
        udp_socket,
        send_addr,
        vec![MSG_OUT_CHANNEL, MARKET_DATA_CHANNEL],
    );

    while let Ok(outbound_engine_message) = rx.recv() {
        multicast_sender.send(outbound_engine_message)
    }

    exit(-1);
}
