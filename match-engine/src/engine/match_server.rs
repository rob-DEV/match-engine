use std::io::BufRead;
use std::sync::mpsc::Sender;

use tokio::io::AsyncReadExt;
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

    async fn order_entry_handler(mut socket: TcpStream, order_tx: Sender<Order>) {
        loop {
            let mut str_buf = String::new();

            let n = socket
                .read_to_string(&mut str_buf)
                .await
                .expect("failed to read data from socket");

            if n == 0 {
                return;
            }

            let mut token_iterator = str_buf.split("|");

            if token_iterator.clone().count() == 3 {
                let side = match token_iterator.next().unwrap().to_uppercase().as_str() {
                    "B" => Side::BUY,
                    "S" => Side::SELL,
                    _ => panic!()
                };

                let qty: u32 = token_iterator.next().unwrap().parse().unwrap();
                let px: u32 = token_iterator.next().unwrap().parse().unwrap();

                let order_for_book = Order::new(1, qty, px, side);
                order_tx.send(order_for_book).unwrap();
            }
        }
    }
}