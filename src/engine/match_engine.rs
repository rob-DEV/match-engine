use std::{
    sync::{mpsc::Receiver, Arc, Mutex},
    thread,
    time::Instant,
};

use super::{
    orderbook::{self, Orderbook},
    types::{Order, Trade},
};

pub struct MatchEngine {
    fifo_orderbook: Arc<Mutex<Orderbook>>,
    executions: Arc<Mutex<Vec<Trade>>>,
}

impl MatchEngine {
    pub fn new() -> MatchEngine {
        MatchEngine {
            fifo_orderbook: Arc::new(Mutex::new(Orderbook::new())),
            executions: Arc::new(Mutex::new(Vec::with_capacity(1000000))),
        }
    }

    pub fn run(&mut self, order_rx: Receiver<Order>) {
        let orderbook_handle: Arc<Mutex<Orderbook>> = self.fifo_orderbook.clone();

        let _order_submission_thread_handle = thread::spawn(move || {
            while let Ok(order_to_book) = order_rx.recv() {
                let mut orderbook = match orderbook_handle.lock() {
                    Ok(orderbook) => orderbook,
                    Err(_) => panic!("Failed to lock executions vector!"),
                };

                orderbook.apply_order(order_to_book)
            }
        });

        let orderbook_handle: Arc<Mutex<Orderbook>> = self.fifo_orderbook.clone();
        let executions_handle: Arc<Mutex<Vec<Trade>>> = self.executions.clone();

        let _match_thread_handle = thread::spawn(move || loop {
            let mut orderbook = match orderbook_handle.lock() {
                Ok(orderbook) => orderbook,
                Err(_) => panic!("Failed to lock executions vector!"),
            };

            let mut executions = match executions_handle.lock() {
                Ok(executions) => executions,
                Err(_) => panic!("Failed to lock executions vector!"),
            };

            let match_cycle_start_time = Instant::now();
            orderbook.check_for_trades(&mut executions);
            let match_cycle_duration_nanos = match_cycle_start_time.elapsed().as_nanos();
            println!("Matching cycle nanos: {}", match_cycle_duration_nanos);
            executions.clear();
        });
    }
}
