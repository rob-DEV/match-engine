use std::{
    sync::{Arc, mpsc::Receiver, Mutex},
    thread,
    time::Instant,
};

use crate::domain::order::Order;
use crate::engine::order_book::OrderBook;

pub struct MatchEngine {
    fifo_orderbook: Arc<Mutex<OrderBook>>,
}

impl MatchEngine {
    pub fn new() -> MatchEngine {
        MatchEngine {
            fifo_orderbook: Arc::new(Mutex::new(OrderBook::new())),
        }
    }

    pub fn run(&self, order_rx: Receiver<Order>) {
        let orderbook_handle: Arc<Mutex<OrderBook>> = self.fifo_orderbook.clone();

        let _order_submission_thread_handle = thread::Builder::new().name("ORDER-SUBMISSION-THREAD".to_owned()).spawn(move || {
            while let Ok(order_to_book) = order_rx.recv() {
                let mut orderbook = match orderbook_handle.lock() {
                    Ok(orderbook) => orderbook,
                    Err(_) => panic!("Failed to lock orderbook!"),
                };

                orderbook.apply_order(order_to_book)
            }
        });

        let orderbook_handle: Arc<Mutex<OrderBook>> = self.fifo_orderbook.clone();

        let _match_thread_handle = thread::Builder::new().name("MATCH-CYCLE-THREAD".to_owned()).spawn(move || MatchEngine::matching_cycle(orderbook_handle));
    }


    fn matching_cycle(orderbook_handle: Arc<Mutex<OrderBook>>) -> ! {
        loop {
            let cycle_timer = Instant::now();
            let mut orderbook = match orderbook_handle.lock() {
                Ok(orderbook) => orderbook,
                Err(_) => panic!("Failed to lock orderbook!"),
            };

            let matches = orderbook.check_for_trades();

            if matches > 0 {
                println!("cycle ns: {} matches: {}", cycle_timer.elapsed().as_nanos(), matches);
            }
        }
    }
}
