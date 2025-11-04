use crate::ENGINE_MSG_IN_PORT;
use common::domain::domain::CancelOrder;
use common::domain::messaging::{EngineMessage, SequencedEngineMessage};
use common::domain::order::{LimitOrder, Order};
use common::network::mutlicast::multicast_receiver;
use common::network::network_constants::MAX_UDP_PACKET_SIZE;
use common::util::time::system_nanos;
use rand::random;
use std::net::UdpSocket;
use std::sync::mpsc::Sender;

pub fn initialize_engine_msg_in_thread(order_entry_tx: Sender<Order>) -> ! {
    println!(
        "Initializing Engine MSG_IN multicast on port {}",
        *ENGINE_MSG_IN_PORT
    );
    let msg_in_socket = multicast_receiver(*ENGINE_MSG_IN_PORT);
    multicast_receiver_to_engine_msg_in(&msg_in_socket, &order_entry_tx);
}

fn multicast_receiver_to_engine_msg_in(udp_socket: &UdpSocket, oe_tx: &Sender<Order>) -> ! {
    let mut buffer = [0; MAX_UDP_PACKET_SIZE];
    let mut last_seen_seq = 0;

    loop {
        let (size, addr) = udp_socket.recv_from(&mut buffer).unwrap();

        let inbound_engine_message: SequencedEngineMessage =
            bitcode::decode(&buffer[..size]).unwrap();

        let ack_bits = inbound_engine_message.sequence_number.to_le_bytes();
        udp_socket
            .send_to(&ack_bits, addr)
            .expect("TODO: panic message");

        if inbound_engine_message.sequence_number != last_seen_seq + 1 {
            eprintln!("Received out of order message");
        }

        last_seen_seq = inbound_engine_message.sequence_number;

        match inbound_engine_message.message {
            EngineMessage::NewOrder(new) => oe_tx
                .send(Order::New(LimitOrder {
                    client_id: new.client_id,
                    id: random::<u32>(),
                    side: new.order_side,
                    px: new.px,
                    qty: new.qty,
                    time_in_force: new.time_in_force,
                    placed_time: system_nanos(),
                }))
                .unwrap(),
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
