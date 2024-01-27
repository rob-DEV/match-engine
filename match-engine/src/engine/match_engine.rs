use std::{
    sync::{Arc, mpsc::Receiver, Mutex},
    thread,
    time::Instant,
};

use crate::domain::order::Order;
use crate::engine::book::Book;

pub struct MatchEngine {
    fifo_book: Arc<Mutex<Book>>,
}

impl MatchEngine {
    pub fn new() -> MatchEngine {
        MatchEngine {
            fifo_book: Arc::new(Mutex::new(Book::new())),
        }
    }

    pub fn run(&self, order_rx: Receiver<Order>) {
        let book_handle_order_entry_thread: Arc<Mutex<Book>> = self.fifo_book.clone();

        let _order_submission_thread_handle = thread::Builder::new().name("ORDER-ENTRY-THREAD".to_owned()).spawn(move || {
            while let Ok(order_to_book) = order_rx.recv() {
                let mut book = match book_handle_order_entry_thread.lock() {
                    Ok(book) => book,
                    Err(_) => panic!("Failed to lock the order book!"),
                };

                book.apply_order(order_to_book)
            }
        });

        let book_handle_cycle_thread: Arc<Mutex<Book>> = self.fifo_book.clone();

        let _match_thread_handle = thread::Builder::new().name("MATCH-CYCLE-THREAD".to_owned()).spawn(move || MatchEngine::matching_cycle(book_handle_cycle_thread));
    }


    fn matching_cycle(book_handle: Arc<Mutex<Book>>) -> ! {
        loop {
            let cycle_timer = Instant::now();
            let mut book = match book_handle.lock() {
                Ok(book) => book,
                Err(_) => panic!("Failed to lock the order book!"),
            };

            let matches = book.check_for_trades();

            if matches > 0 {
                println!("cycle ns: {} matches: {}", cycle_timer.elapsed().as_nanos(), matches);
            }
        }
    }
}
