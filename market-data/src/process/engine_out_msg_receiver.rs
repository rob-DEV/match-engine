use common::network::mutlicast::multicast_receiver;
use common::transport::nack_sequenced_multicast_receiver::NackSequencedMulticastReceiver;
use std::error::Error;
use lazy_static::lazy_static;

pub fn initialize_engine_msg_out_receiver(
    engine_msg_out_port: u16,
) -> Result<(), Box<dyn Error>> {
    let udp_socket = multicast_receiver(engine_msg_out_port);
    let mut multicast_receiver = NackSequencedMulticastReceiver::new(udp_socket, 9001);

    println!(
        "Initialized MSG_OUT -> MDD multicast on port {}",
        engine_msg_out_port
    );

    loop {
        if let Some(outbound_engine_message) = multicast_receiver.try_recv() {

        }
    }

    Ok(())
}
