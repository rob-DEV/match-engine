use std::{
    sync::{Arc, Mutex},
    thread,
    time::Instant,
};

use engine::{orderbook::Orderbook, types::Trade, match_engine::MatchEngine};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
};

use crate::engine::types::{Order, Side};

mod engine;

#[tokio::main]
async fn main() {
    let client_connection_listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();

    let engine_mutex = Arc::new(Mutex::new(engine::match_engine::MatchEngine::new()));

    let mut executed_trades: Vec<Trade> = Vec::with_capacity(1000000);

    let engine_mutex_match_thread = engine_mutex.clone();

    let _handle = thread::spawn(move || loop {
        match engine_mutex_match_thread.lock() {
            Ok(mut orderbook) => {
                orderbook.run();
            }
            Err(_) => println!("Error locking engine"),
        }
    });

    loop {
        let engine_mutex = engine_mutex.clone();
        let (socket, _) = client_connection_listener.accept().await.unwrap();

        tokio::spawn(async move {
            client_connection_handler(socket, engine_mutex).await;
        });
    }
}

async fn client_connection_handler(mut tcp_stream: TcpStream, engine: Arc<Mutex<MatchEngine>>) {
    println!("Connection Established");

    let (input_stream, mut output_stream) = tcp_stream.split();

    let mut reader = BufReader::new(input_stream);
    let mut line = String::new();

    output_stream
        .write_all(
            "Place your order in the format:\n[B/S],Qty,Px\n"
                .to_owned()
                .as_bytes(),
        )
        .await
        .unwrap();

    while let Ok(stream_bytes_read) = reader.read_line(&mut line).await {
        if stream_bytes_read == 0 {
            break;
        }

        let trimmed_input = line.trim().to_owned();
        println!("Input: {:?}", trimmed_input.as_bytes());

        let mut token_iterator = trimmed_input.split(",");

        // Dodgy parse the BID and OFFER
        if token_iterator.clone().count() == 3 {
            let side = match token_iterator.next().unwrap().to_uppercase().as_str() {
                "B" => Side::BUY,
                "S" => Side::SELL,
                _ => panic!(),
            };

            let qty = token_iterator.next().unwrap().parse::<u32>().unwrap();
            let px = token_iterator.next().unwrap().parse::<u32>().unwrap();

            let order_for_book = Order::new(1, 1, qty, px, side);

            let mut ob = engine.lock().unwrap();
            ob.apply_order(order_for_book);
        }

        line.clear();
    }

    println!("Connection closed!");
}

#[cfg(test)]
mod tests {
    use crate::engine::orderbook::Orderbook;
    use crate::engine::types::Side::{BUY, SELL};
    use crate::engine::types::{Order, Trade};
    use std::time::Instant;

    #[test]
    fn order_book_test() {
        let mut orderbook = Orderbook::new();
        let mut executed_trades: Vec<Trade> = Vec::with_capacity(1000000);

        // use rand::Rng;
        // for _ in 1..10 {
        //     let mut rng = rand::thread_rng();
        //     let user_id = rng.gen_range(1000..100000);
        //     let price = rng.gen_range(1..1000);
        //     let quantity = rng.gen_range(1..50);
        //     let buy_order = Order::new(user_id, 1, price, quantity, BUY);

        //     let user_id = rng.gen_range(1000..100000);
        //     let price = rng.gen_range(1..1000);
        //     let quantity = rng.gen_range(1..50);
        //     let sell_order = Order::new(user_id, 1, price, quantity, SELL);

        //     orderbook.apply_order(buy_order);
        //     orderbook.apply_order(sell_order);
        // }

        orderbook.apply_order(Order::new(1, 1, 100, 10, BUY));
        orderbook.apply_order(Order::new(2, 1, 100, 5, SELL));
        orderbook.apply_order(Order::new(3, 1, 100, 10, SELL));

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
}
