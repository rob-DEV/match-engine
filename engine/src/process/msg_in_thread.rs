use crate::domain::order::{LimitOrder, Order};
use crate::ENGINE_MSG_IN_PORT;
use common::message::cancel_order::CancelOrder;
use common::network::mutlicast::multicast_receiver;
use common::transport::sequenced_message::EngineMessage;
use common::transport::sequenced_multicast_receiver::SequencedMulticastReceiver;
use common::transport::transport_constants::MSG_IN_CHANNEL;
use common::util::time::system_nanos;
use std::net::UdpSocket;
use std::sync::mpsc::Sender;

pub fn initialize_engine_msg_in_thread(order_entry_tx: Sender<Order>) -> ! {
    println!(
        "Initializing Engine MSG_IN multicast on port {}",
        *ENGINE_MSG_IN_PORT
    );
    let msg_in_socket = multicast_receiver(*ENGINE_MSG_IN_PORT);

    multicast_receiver_to_engine_msg_in(Box::new(msg_in_socket), &order_entry_tx);
}

fn multicast_receiver_to_engine_msg_in(udp_socket: Box<UdpSocket>, oe_tx: &Sender<Order>) -> ! {
    let mut last_seen_seq = 0;

    let mut multicast_receiver = SequencedMulticastReceiver::new(udp_socket, MSG_IN_CHANNEL);

    let mut init_oe_seq = 1000;

    loop {
        if let Some(inbound_engine_message) = multicast_receiver.try_recv() {
            if inbound_engine_message.sequence_number != last_seen_seq + 1 {
                eprintln!("Received out of order message");
            }

            last_seen_seq = inbound_engine_message.sequence_number;

            match inbound_engine_message.message {
                EngineMessage::NewOrder(new) => {
                    oe_tx
                        .send(Order::LimitOrder(LimitOrder {
                            client_id: new.client_id,
                            id: init_oe_seq,
                            side: new.order_side,
                            px: new.px,
                            qty: new.qty,
                            time_in_force: new.time_in_force,
                            placed_time: system_nanos(),
                        }))
                        .unwrap();
                    init_oe_seq += 1;
                }
                EngineMessage::CancelOrder(cancel) => oe_tx
                    .send(Order::Cancel(CancelOrder {
                        client_id: cancel.client_id,
                        order_side: cancel.order_side,
                        order_id: cancel.order_id,
                    }))
                    .unwrap(),
                _ => {
                    unimplemented!()
                }
            }
        }
    }
}
