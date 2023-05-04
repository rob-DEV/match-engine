use std::time::Instant;
use rand::Rng;
use crate::engine::orderbook::Orderbook;
use crate::engine::types::Order;
use crate::engine::types::Side::{BUY, SELL};

mod engine;

fn main() {
    let mut rng = rand::thread_rng();
    let mut orderbook = Orderbook::new();

    for _ in 0..10000 {
        let user_id = rng.gen_range(1000..100000);
        let price = rng.gen_range(1..50);
        let quantity = rng.gen_range(1..50);
        let buy_order = Order::new(user_id, 1, price, quantity, BUY);

        let user_id = rng.gen_range(1000..100000);
        let price = rng.gen_range(1..50);
        let quantity = rng.gen_range(1..50);
        let sell_order = Order::new(user_id, 1, price, quantity, SELL);

        orderbook.apply_order(buy_order);
        orderbook.apply_order(sell_order);
    }

    // let buy_order = Order::new(001, 1, 100, 2, BUY);
    // let sell_order = Order::new(002, 1, 100, 10, SELL);
    // orderbook.apply_order(buy_order);
    // orderbook.apply_order(sell_order);

    let match_cycle_start_time = Instant::now();
    let trades = orderbook.check_for_trades();

    let match_cycle_duration_micros =  match_cycle_start_time.elapsed().as_micros();
    println!("Matched Trades: {}", trades.len());
    println!("Matching cycle duration: {} microseconds", match_cycle_duration_micros);
}