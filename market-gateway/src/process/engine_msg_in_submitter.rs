use crate::message::GatewayMessage;
use crate::ENGINE_MSG_IN_PORT;
use common::network::mutlicast::multicast_sender;
use common::transport::nack_sequenced_multicast_sender::NackSequencedMulticastSender;
use common::transport::sequenced_message::EngineMessage;
use std::error::Error;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use tokio::sync::mpsc::UnboundedReceiver;

pub fn initialize_engine_msg_in_message_submitter(
    mut rx: UnboundedReceiver<GatewayMessage>,
) -> Result<(), Box<dyn Error>> {
    let udp_socket = multicast_sender();
    let send_addr = SocketAddr::V4(SocketAddrV4::new(
        Ipv4Addr::new(239, 255, 0, 1),
        *ENGINE_MSG_IN_PORT,
    ));

    let mut multicast_sender = NackSequencedMulticastSender::new(udp_socket, send_addr, 9000);

    println!(
        "Initialized Gateway -> MSG_IN multicast on port {}",
        *ENGINE_MSG_IN_PORT
    );

    loop {
        while let Ok(inbound_engine_message) = rx.try_recv() {
            let message_in = match inbound_engine_message {
                GatewayMessage::LimitOrder(new) => EngineMessage::NewOrder(new),
                GatewayMessage::MarketOrder(_) => unimplemented!(),
                GatewayMessage::CancelOrder(cancel) => EngineMessage::CancelOrder(cancel),
            };

            multicast_sender.send(message_in);
        }
    }
}
