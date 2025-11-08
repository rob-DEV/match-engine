use crate::message::GatewayMessage;
use crate::ENGINE_MSG_IN_PORT;
use common::network::mutlicast::multicast_sender;
use common::transport::sequenced_message::EngineMessage;
use common::transport::sequenced_multicast_sender::SequencedMulticastSender;
use common::transport::transport_constants::MSG_IN_CHANNEL;
use std::error::Error;
use std::net::SocketAddr;
use tokio::sync::mpsc::UnboundedReceiver;

pub fn initialize_engine_msg_in_message_submitter(
    mut rx: UnboundedReceiver<GatewayMessage>,
) -> Result<(), Box<dyn Error>> {
    let udp_socket = multicast_sender();
    let send_addr = "239.255.0.1:3000".parse::<SocketAddr>().unwrap();

    let mut multicast_sender =
        SequencedMulticastSender::new(Box::new(udp_socket), send_addr, vec![MSG_IN_CHANNEL]);

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
