mod md_book;

use crate::md_book::MarketDataBook;
use common::network::mutlicast::multicast_receiver;
use common::transport::nack_sequenced_multicast_receiver::NackSequencedMulticastReceiver;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref ENGINE_MSG_OUT_PORT: u16 = 3500;
}

fn main() {
    let udp_socket = multicast_receiver(*ENGINE_MSG_OUT_PORT);

    let mut multicast_receiver = NackSequencedMulticastReceiver::new(udp_socket, 9001);

    println!(
        "Initialized MSG_OUT -> Market Data Reporter multicast on port {}",
        *ENGINE_MSG_OUT_PORT
    );

    let mut last_seen_seq = 0;
    let mut market_data_book = MarketDataBook::new();

    loop {
        if let Some(outbound_engine_message) = multicast_receiver.try_recv() {
            last_seen_seq += 1;
            let outbound_engine_message = &outbound_engine_message.message;

            let updated = market_data_book.update_from_engine(outbound_engine_message);

            // market_data_book.emit_l1();
            if updated && last_seen_seq % 100_000 == 0 {
                market_data_book.emit_l2();
                println!("{}", market_data_book.order_count())
            }
        }
    }
}
