use std::time::Instant;

use crate::engine::orderbook::Orderbook;
use crate::engine::types::Side::{BUY, SELL};
use crate::engine::types::{Order, Trade};

mod engine;

fn main() {
    let mut orderbook = Orderbook::new();
    let mut executed_trades: Vec<Trade> = Vec::with_capacity(1000000);

    // let buy_order = Order::new(001, 1, 100, 2, BUY);
    // let sell_order = Order::new(002, 1, 101, 10, SELL);
    // let matching_buy_for_sell_order = Order::new(003, 1, 101, 4, BUY);
    // orderbook.apply_order(buy_order);
    // orderbook.apply_order(sell_order);
    // orderbook.apply_order(matching_buy_for_sell_order);

    use rand::Rng;
    for _ in 1..20 {
        let mut rng = rand::thread_rng();
        let user_id = rng.gen_range(1000..100000);
        let price = rng.gen_range(1..5000);
        let quantity = rng.gen_range(1..50);
        let buy_order = Order::new(user_id, 1, price, quantity, BUY);

        let user_id = rng.gen_range(1000..100000);
        let price = rng.gen_range(1..5000);
        let quantity = rng.gen_range(1..50);
        let sell_order = Order::new(user_id, 1, price, quantity, SELL);

        orderbook.apply_order(buy_order);
        orderbook.apply_order(sell_order);
    }

    let match_cycle_start_time = Instant::now();
    orderbook.check_for_trades(&mut executed_trades);
    let match_cycle_duration_micros = match_cycle_start_time.elapsed().as_micros();

    println!(
        "Matching cycle duration: {} microseconds",
        match_cycle_duration_micros
    );

    println!("Matched Trades: {}", executed_trades.len());
    println!("Trades:");
    println!(
        "{0: <3} | {1: <5} | {2: <5} | {3: <4}",
        "Qty", "Px", "B/CId", "S/CId"
    );

    for trade in &executed_trades {
        println!("{:?}", trade);
    }

    println!("{:?}", orderbook);

    executed_trades.clear();
}
