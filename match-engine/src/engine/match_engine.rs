use std::{sync::{Arc, mpsc::Receiver, Mutex}, thread};
use std::time::Instant;

use common::message::MarketDataFullSnapshot;

use crate::domain::execution::Execution;
use crate::domain::order::Order;
use crate::engine::clob::CentralLimitOrderBook;
use crate::memory::util::uninitialized_arr;

const MAX_EXECUTIONS_PER_CYCLE: usize = 1000;

pub struct MatchEngine {
    book_mutex: Arc<Mutex<CentralLimitOrderBook>>,
}

impl MatchEngine {
    pub fn new(market_data_tx: Arc<Mutex<MarketDataFullSnapshot>>) -> MatchEngine {
        println!("Initializing Match Engine");

        MatchEngine {
            book_mutex: Arc::new(Mutex::new(CentralLimitOrderBook::new(market_data_tx)))
        }
    }

    pub fn run(&self, order_rx: Receiver<Order>) {
        let book_mutex: Arc<Mutex<CentralLimitOrderBook>> = self.book_mutex.clone();

        let _order_submission_thread_handle = thread::Builder::new()
            .name("ORDER-ENTRY-THREAD".to_owned())
            .spawn(move || Self::order_entry(book_mutex, order_rx));

        let book_handle_cycle_thread = self.book_mutex.clone();

        let _match_thread_handle = thread::Builder::new()
            .name("MATCH-CYCLE-THREAD".to_owned())
            .spawn(move || Self::matching_cycle(book_handle_cycle_thread));
    }

    fn order_entry(book_mutex: Arc<Mutex<CentralLimitOrderBook>>, order_tx: Receiver<Order>) {
        while let Ok(order) = order_tx.recv() {
            let mut book = book_mutex.lock().unwrap();
            match order {
                Order::New(new_order) => book.apply_order(new_order),
                Order::Cancel(cancel_order) => book.remove_order(cancel_order)
            }
        }
    }

    fn matching_cycle(book_handle: Arc<Mutex<CentralLimitOrderBook>>) -> ! {
        let mut executions_buffer = uninitialized_arr::<Execution, MAX_EXECUTIONS_PER_CYCLE>();

        loop {
            let cycle_timer = Instant::now();
            let mut book = book_handle.lock().unwrap();
            let executions = book.check_for_trades(MAX_EXECUTIONS_PER_CYCLE, &mut executions_buffer);

            if executions > 0 {
                println!("cycle ns: {} executions: {}", cycle_timer.elapsed().as_nanos(), executions);
                book.populate_md_mutex();
            }
        }
    }
}
