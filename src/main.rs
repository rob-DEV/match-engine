use matching_engine::initialize;
use matching_engine::engine::OrderMatchType;
use matching_engine::engine::Order;
use matching_engine::engine::BuyOrSell;
use matching_engine::engine::Orderbook;
use matching_engine::engine::MatchEngine;

mod matching_engine;

use rand::Rng;

fn main() {
    let mut rng = rand::thread_rng();
    let mut orderbook = Orderbook::new();

    // Bids
    for i in 0..1000 {
        let user_id = rng.gen_range(1000..100000);
        let quantity = rng.gen_range(1..50);
        let price = rng.gen_range(1..50);
        let order = Order::new(user_id, quantity, price, BuyOrSell::BUY, OrderMatchType::AON);

        orderbook.place_order(order);
    }

    // Offers
    for i in 0..1000 {
        let user_id = rng.gen_range(1000..100000);
        let quantity = rng.gen_range(1..50);
        let price = rng.gen_range(1..50);
        let order = Order::new(user_id, quantity, price, BuyOrSell::SELL, OrderMatchType::AON);
        
        orderbook.place_order(order);
    }

    let mut engine = MatchEngine::new(orderbook);

    use std::time::Instant;
    let now = Instant::now();

    let executed_orders = engine.cycle();

    let elapsed = now.elapsed();
    // println!("Executions: {:?}", executed_orders);
    println!("Elapsed: {:.2?}", elapsed);



    initialize::initialize();
}
