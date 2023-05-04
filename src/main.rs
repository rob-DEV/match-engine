use std::time::Instant;
use rand::Rng;
use crate::engine::orderbook::Orderbook;
use crate::engine::types::Order;
use crate::engine::types::Side::{BUY, SELL};

mod engine;

fn main() {
    let mut orderbook = Orderbook::new();

    println!("---------------1st Cycle with [BUY-2@100, SELL-10@101] orders added---------------");
    let buy_order = Order::new(001, 1, 100, 2, BUY);
    let sell_order = Order::new(002, 1, 101, 10, SELL);
    orderbook.apply_order(buy_order);
    orderbook.apply_order(sell_order);

    let match_cycle_start_time = Instant::now();
    let trades = orderbook.check_for_trades();

    let match_cycle_duration_micros =  match_cycle_start_time.elapsed().as_micros();
    println!("Matched Trades: {}", trades.len());
    println!("Matching cycle duration: {} microseconds", match_cycle_duration_micros);

    println!("{:?}", orderbook);

    println!("---------------2nd Cycle with BUY-4@101 order added---------------");
    let matching_buy_for_sell_order = Order::new(003, 1, 101, 4, BUY);
    orderbook.apply_order(matching_buy_for_sell_order);

    println!("{:?}", orderbook);

    let match_cycle_start_time = Instant::now();
    let trades = orderbook.check_for_trades();

    let match_cycle_duration_micros =  match_cycle_start_time.elapsed().as_micros();
    println!("Matched Trades: {}", trades.len());
    println!("Matching cycle duration: {} microseconds", match_cycle_duration_micros);

    println!("{:?}", orderbook);
}