mod md_book;

use crate::md_book::MarketDataBook;
use common::network::mutlicast::multicast_receiver;
use common::network::network_constants::MAX_UDP_PACKET_SIZE;
use common::transport::sequenced_message::SequencedEngineMessage;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref ENGINE_MSG_OUT_PORT: u16 = 3500;
}

fn main() {
    let udp_socket = multicast_receiver(*ENGINE_MSG_OUT_PORT);
    let mut buffer = [0; MAX_UDP_PACKET_SIZE];

    println!(
        "Initialized MSG_OUT -> Market Data Reporter multicast on port {}",
        *ENGINE_MSG_OUT_PORT
    );

    let mut market_data_book = MarketDataBook::new();

    loop {
        match udp_socket.recv_from(&mut buffer) {
            Ok((size, _)) => {
                let outbound_engine_message: SequencedEngineMessage =
                    bitcode::decode(&buffer[..size]).unwrap();

                let outbound_engine_message = &outbound_engine_message.message;

                market_data_book.update_from_engine(outbound_engine_message);


                market_data_book.emit_l1();
                market_data_book.emit_l2();
            }
            Err(_) => {}
        }
    }
}
