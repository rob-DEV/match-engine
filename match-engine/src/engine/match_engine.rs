use std::{
    sync::{Arc, mpsc::Receiver, Mutex},
    thread,
    time::Instant,
};

use common::message::MarketDataFullSnapshot;

use crate::domain::order::{Order, OrderType};
use crate::engine::book::Book;

pub struct MatchEngine {
    book_mutex: Arc<Mutex<Book>>,
    order_rx_mutex: Arc<Mutex<Receiver<Order>>>,
    market_data_tx_mutex: Arc<Mutex<MarketDataFullSnapshot>>,
}

impl MatchEngine {
    pub fn new(order_rx: Receiver<Order>, market_data_tx: Arc<Mutex<MarketDataFullSnapshot>>) -> MatchEngine {
        MatchEngine {
            book_mutex: Arc::new(Mutex::new(Book::new())),
            order_rx_mutex: Arc::new(Mutex::new(order_rx)),
            market_data_tx_mutex: market_data_tx,
        }
    }

    pub fn run(&self) {
        let book_mutex: Arc<Mutex<Book>> = self.book_mutex.clone();
        let order_tx_mutex: Arc<Mutex<Receiver<Order>>> = self.order_rx_mutex.clone();

        let _order_submission_thread_handle = thread::Builder::new().name("ORDER-ENTRY-THREAD".to_owned()).spawn(move || {
            while let Ok(order) = order_tx_mutex.lock().unwrap().recv() {
                let mut book = book_mutex.lock().unwrap();
                match order.order_type {
                    OrderType::New => book.apply_order(order),
                    OrderType::Cancel => book.remove_order(order)
                }
            }
        });

        let book_handle_cycle_thread = self.book_mutex.clone();
        let market_data_tx_mutex: Arc<Mutex<MarketDataFullSnapshot>> = self.market_data_tx_mutex.clone();

        let _match_thread_handle = thread::Builder::new().name("MATCH-CYCLE-THREAD".to_owned()).spawn(move || MatchEngine::matching_cycle(book_handle_cycle_thread, market_data_tx_mutex));
    }


    fn matching_cycle(book_handle: Arc<Mutex<Book>>, md_mutex: Arc<Mutex<MarketDataFullSnapshot>>) -> ! {
        loop {
            let cycle_timer = Instant::now();
            let mut book = book_handle.lock().unwrap();

            let matches = book.check_for_trades();

            if matches > 0 {
                println!("cycle ns: {} matches: {}", cycle_timer.elapsed().as_nanos(), matches);
            }

            let book_snapshot = book.create_book_snapshot();
            let mut snapshot = md_mutex.lock().unwrap();
            snapshot.asks = book_snapshot.asks;
            snapshot.bids = book_snapshot.bids;
        }
    }
}
