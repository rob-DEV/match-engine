use std::{
    sync::{Arc, mpsc::Receiver, Mutex},
    thread,
    time::Instant,
};

use super::{
    domain::{Order, Trade},
    order_book::OrderBook,
};

pub struct MatchEngine {
    fifo_orderbook: Arc<Mutex<OrderBook>>,
    executions: Arc<Mutex<Vec<Trade>>>,
}

impl MatchEngine {
    pub fn new() -> MatchEngine {
        MatchEngine {
            fifo_orderbook: Arc::new(Mutex::new(OrderBook::new())),
            executions: Arc::new(Mutex::new(Vec::with_capacity(1000000))),
        }
    }

    pub fn run(&self, order_rx: Receiver<Order>) {
        let orderbook_handle: Arc<Mutex<OrderBook>> = self.fifo_orderbook.clone();

        let _order_submission_thread_handle = thread::Builder::new().name("MATCH-THREAD".to_owned()).spawn(move || {
            while let Ok(order_to_book) = order_rx.recv() {
                let mut orderbook = match orderbook_handle.lock() {
                    Ok(orderbook) => orderbook,
                    Err(_) => panic!("Failed to lock orderbook!"),
                };

                orderbook.apply_order(order_to_book)
            }
        });

        let orderbook_handle: Arc<Mutex<OrderBook>> = self.fifo_orderbook.clone();
        let executions_handle: Arc<Mutex<Vec<Trade>>> = self.executions.clone();

        let _match_thread_handle = thread::spawn(move || MatchEngine::matching_cycle(orderbook_handle, executions_handle));
    }


    fn matching_cycle(orderbook_handle: Arc<Mutex<OrderBook>>, executions_handle: Arc<Mutex<Vec<Trade>>>) -> ! {
        let mut cycle_timer = Instant::now();
        let mut cycle_count: u32 = 0;
        loop {
            let mut orderbook = match orderbook_handle.lock() {
                Ok(orderbook) => orderbook,
                Err(_) => panic!("Failed to lock orderbook!"),
            };

            let mut executions = match executions_handle.lock() {
                Ok(executions) => executions,
                Err(_) => panic!("Failed to lock executions vector!"),
            };

            orderbook.check_for_trades(&mut executions);

            cycle_count = cycle_count + 1;
            if cycle_timer.elapsed().as_millis() > 1000 {
                println!("Matching cycles per seconds: {}", cycle_count);
                cycle_timer = Instant::now();
                cycle_count = 0;
            }

            if executions.len() > 0 {
                println!("Matched trades: {}", executions.len());
            }

            executions.clear();
        }
    }
}
