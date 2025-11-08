mod md_book;

use crate::md_book::MarketDataBook;
use common::network::mutlicast::multicast_receiver;
use common::transport::sequenced_multicast_receiver::SequencedMulticastReceiver;
use common::transport::transport_constants::MARKET_DATA_CHANNEL;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref ENGINE_MSG_OUT_PORT: u16 = 3500;
}

fn main() {
    let udp_socket = multicast_receiver(*ENGINE_MSG_OUT_PORT);

    let mut multicast_receiver =
        SequencedMulticastReceiver::new(Box::from(udp_socket), MARKET_DATA_CHANNEL);

    println!(
        "Initialized MSG_OUT -> Market Data Reporter multicast on port {}",
        *ENGINE_MSG_OUT_PORT
    );

    let mut last_seen_seq = 0;
    let mut market_data_book = MarketDataBook::new();

    let mut emit_rate = 0;
    loop {
        if let Some(outbound_engine_message) = multicast_receiver.try_recv() {
            assert_eq!(outbound_engine_message.sequence_number, last_seen_seq + 1);
            last_seen_seq += 1;
            let outbound_engine_message = &outbound_engine_message.message;

            market_data_book.update_from_engine(outbound_engine_message);

            // market_data_book.emit_l1();
            if emit_rate % 1 == 0 {
                market_data_book.emit_l2();
            }

            emit_rate += 1;
        }
    }
}
