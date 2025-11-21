use crate::domain::limit_order::LimitOrder;
use crate::domain::order::Order;
use common::network::mutlicast::multicast_receiver;
use common::transport::nack_sequenced_multicast_receiver::NackSequencedMulticastReceiver;
use common::transport::sequenced_message::EngineMessage;
use common::types::cancel_order::CancelOrderRequest;
use common::util::time::system_nanos;
use core_affinity::CoreId;
use std::sync::mpsc::Sender;
use std::thread;
use std::thread::JoinHandle;

pub fn msg_in_thread(
    msg_in_port: u16,
    order_entry_tx: Sender<Order>,
    pinned_msg_in_core: CoreId,
) -> JoinHandle<()> {
    thread::spawn(move || {
        core_affinity::set_for_current(pinned_msg_in_core);
        let msg_in_socket = multicast_receiver(msg_in_port);

        let mut last_seen_seq = 0;

        let mut multicast_receiver = NackSequencedMulticastReceiver::new(msg_in_socket, 9000);

        let mut init_oe_seq = 1000;

        loop {
            if let Some(inbound_engine_message) = multicast_receiver.try_recv() {
                if inbound_engine_message.sequence_number != last_seen_seq + 1 {
                    eprintln!(
                        "Received out of order message actual: {} expected: {}",
                        inbound_engine_message.sequence_number, last_seen_seq
                    );
                }

                last_seen_seq += 1;

                match inbound_engine_message.message {
                    EngineMessage::NewOrder(new) => {
                        order_entry_tx
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
                    EngineMessage::CancelOrder(cancel) => order_entry_tx
                        .send(Order::Cancel(CancelOrderRequest {
                            client_id: cancel.client_id,
                            order_side: cancel.order_side,
                            order_id: cancel.order_id,
                            instrument: [0; 16],
                        }))
                        .unwrap(),
                    _ => {
                        unimplemented!()
                    }
                }
            }
        }
    })
}
