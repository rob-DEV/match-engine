use common::network::mutlicast::multicast_receiver;
use common::transport::nack_sequenced_multicast_receiver::NackSequencedMulticastReceiver;
use common::transport::sequenced_message::EngineMessage;
use core_affinity::CoreId;
use std::thread;
use std::thread::JoinHandle;
use tokio::sync::broadcast::Sender;

pub fn msg_out_thread(
    msg_out_port: u16,
    pinned_msg_out_core: CoreId,
    tx_engine_queue: Sender<EngineMessage>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        // core_affinity::set_for_current(pinned_msg_out_core);
        let udp_socket = multicast_receiver(msg_out_port);

        let mut multicast_receiver = NackSequencedMulticastReceiver::new(udp_socket, 9001);

        loop {
            if let Some(outbound_engine_message) = multicast_receiver.try_recv() {
                tx_engine_queue
                    .send(outbound_engine_message.message)
                    .unwrap();
            }
        }
    })
}
