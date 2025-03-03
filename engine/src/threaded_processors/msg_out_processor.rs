use crate::internal::order::{CancelOrder, LimitOrder, Order};
use common::domain::messaging::{EngineMessage, InboundEngineMessage, OutboundEngineMessage};
use common::util::time::epoch_nanos;
use rand::random;
use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc::{Receiver, Sender};


pub fn multicast_receiver_to_engine_msg_in(mut udp_socket: &UdpSocket, mut oe_tx: &Sender<Order>) -> ! {
    let mut buffer = [0; 32768];

    loop {
        let (size, _) = udp_socket.recv_from(&mut buffer).unwrap();
        let inbound_engine_message: Vec<InboundEngineMessage> = bitcode::decode(&buffer[..size]).unwrap();
        for msg_in in inbound_engine_message {
            match msg_in.message {
                EngineMessage::NewOrder(new) => {
                    oe_tx.send(Order::New(LimitOrder {
                        client_id: new.client_id,
                        id: random::<u32>(),
                        action: new.order_action,
                        px: new.px,
                        qty: new.qty,
                        placed_time: epoch_nanos(),
                    })).unwrap()
                }
                EngineMessage::CancelOrder(cancel) => {
                    oe_tx.send(Order::Cancel(CancelOrder {
                        client_id: cancel.client_id,
                        action: cancel.order_action,
                        id: cancel.order_id,
                    })).unwrap()
                }
                _ => {
                    unimplemented!()
                }
            }
        }
    }
}

pub fn engine_msg_out_to_multicast(mut rx: &Receiver<OutboundEngineMessage>, mut udp_socket: &UdpSocket) -> ! {
    let send_addr = "0.0.0.0:3500".parse::<SocketAddr>().unwrap();

    while let Ok(outbound_engine_message) = rx.recv() {
        let encoded: Vec<u8> = bitcode::encode(&outbound_engine_message);
        udp_socket.send_to(&encoded, send_addr).expect("TODO: panic message");
    }

    loop {}
}
