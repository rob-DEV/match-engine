use std::sync::mpsc::Sender;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

use crate::engine::domain::{Order, Side};

pub struct MatchServer {
    server: TcpListener,
}

impl MatchServer {
    pub async fn new() -> MatchServer {
        MatchServer {
            server: TcpListener::bind("127.0.0.1:8080").await.unwrap()
        }
    }

    pub async fn run(&self, order_entry_tx: Sender<Order>) {
        loop {
            let order = order_entry_tx.clone();
            let (socket, _) = self.server.accept().await.unwrap();

            tokio::spawn(async move {
                MatchServer::order_entry_handler(socket, order).await;
            });
        }
    }

    async fn order_entry_handler(mut tcp_stream: TcpStream, order_tx: Sender<Order>) {
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

                let order_for_book = Order::new(1, qty, px, side);

                order_tx.send(order_for_book).unwrap();
            }

            line.clear();
        }
    }
}
