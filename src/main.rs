use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
};

mod engine;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        // A new task is spawned for each inbound socket. The socket is
        // moved to the new task and processed there.
        tokio::spawn(async move {
            process(socket).await;
        });
    }

    //client - send random orders every second
    // server receive order, sequence, add to book
}

async fn process(mut tcp_stream: TcpStream) {
    println!("Connection Established");

    let (reader, mut writer) = tcp_stream.split();

    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    while let Ok(bytes_read_from_stream) = reader.read_line(&mut line).await {
        if bytes_read_from_stream == 0 {
            break;
        }

        println!("Input: {:?}", line.trim().as_bytes());

        line.clear();
        writer
            .write_all("Response\n".to_owned().as_bytes())
            .await
            .unwrap();
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
