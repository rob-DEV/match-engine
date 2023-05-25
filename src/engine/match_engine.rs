use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use super::{
    orderbook::Orderbook,
    types::{Order, Trade},
};

pub struct MatchEngine {
    fifo_orderbook: Orderbook,
    executions: Vec<Trade>
}

impl MatchEngine {
    pub fn new() -> MatchEngine {
        MatchEngine {
            fifo_orderbook: Orderbook::new(),
            executions: Vec::with_capacity(1000000),
        }
    }

    pub fn apply_order(&mut self, order: Order) {
        self.fifo_orderbook.apply_order(order);
    }

    pub fn run(&mut self) {
        let match_cycle_start_time = Instant::now();
        self.fifo_orderbook.check_for_trades(&mut self.executions);
        let _match_cycle_duration_micros = match_cycle_start_time.elapsed().as_micros();

        if self.executions.len() > 0 {
            println!("Matched Trades: {}", self.executions.len());
            println!("Trades:");
            println!(
                "{0: <3} | {1: <5} | {2: <5} | {3: <4}",
                "Qty", "Px", "B/CId", "S/CId"
            );

            for trade in self.executions.iter() {
                println!("{:?}", trade);
            }

            self.executions.clear();
            println!("{:?}", self.fifo_orderbook);
        }
    }
}
