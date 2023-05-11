use std::net::SocketAddr;

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpListener, sync::broadcast,
};

mod engine;

#[tokio::main]
async fn main() {
    let tcp_listener = match TcpListener::bind("localhost:8080").await {
        Ok(listener) => listener,
        Err(error) => panic!("Error establishing tcp listener: {:?}", error),
    };

    let (tx, _) = broadcast::channel::<(String, SocketAddr)>(10);

    // Multiple connections
    loop {
        let (mut socket, addr) = tcp_listener.accept().await.unwrap();

        let tx = tx.clone();
        let mut rx = tx.subscribe();

        tokio::spawn(async move  {
            let (reader, mut writer) = socket.split();

            let mut reader = BufReader::new(reader);
            let mut line = String::new();
            
            loop {
                tokio::select! {
                    result = reader.read_line(&mut line) => {
                        if result.unwrap() == 0 {
                            break;
                        }
                        tx.send((line.clone(), addr)).unwrap();
                        line.clear();
                    }
                    result = rx.recv() => {
                        let (msg, other_addr) = result.unwrap();

                        if (addr != other_addr) {
                            writer.write_all(msg.as_bytes()).await.unwrap();
                        }
                    }
                }
            }
        });
    }
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

        use rand::Rng;
        for _ in 1..10 {
            let mut rng = rand::thread_rng();
            let user_id = rng.gen_range(1000..100000);
            let price = rng.gen_range(1..1000);
            let quantity = rng.gen_range(1..50);
            let buy_order = Order::new(user_id, 1, price, quantity, BUY);

            let user_id = rng.gen_range(1000..100000);
            let price = rng.gen_range(1..1000);
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
}
