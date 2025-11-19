use common::network::mutlicast::multicast_receiver;
use common::transport::nack_sequenced_multicast_receiver::NackSequencedMulticastReceiver;
use common::transport::sequenced_message::SequencedEngineMessage;
use std::error::Error;
use tokio::sync::mpsc;

pub fn initialize_engine_msg_out_receiver(
    engine_msg_out_port: u16,
    tx: mpsc::Sender<SequencedEngineMessage>,
) -> Result<(), Box<dyn Error>> {
    let udp_socket = multicast_receiver(engine_msg_out_port);
    let mut multicast_receiver = NackSequencedMulticastReceiver::new(udp_socket, 9001);

    println!(
        "Initialized MSG_OUT -> MDD multicast on port {}",
        engine_msg_out_port
    );

    loop {
        if let Some(outbound_engine_message) = multicast_receiver.try_recv() {
            if tx.blocking_send(outbound_engine_message).is_err() {
                eprintln!("mpsc channel closed");
                break;
            }
        }
    }

    Ok(())
}
