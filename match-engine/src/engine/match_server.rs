use std::sync::mpsc::Sender;
use std::time::{SystemTime, UNIX_EPOCH};

use rand::random;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use common::message::{GatewayMessage, NewOrder, NewOrderAck, TradeAction};

use crate::domain::order::Order;
use crate::domain::side::Side;

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
            let order_tx = order_entry_tx.clone();
            let (socket, _) = self.server.accept().await.unwrap();

            tokio::spawn(async move {
                MatchServer::handler(socket, order_tx).await;
            });
        }
    }

    async fn handler(mut socket: TcpStream, order_tx: Sender<Order>) {
        let (mut rx, mut tx) = socket.split();
        let mut buf: [u8; 512] = [0; 512];

        match rx.read(&mut buf).await {
            Ok(bytes) => {
                if bytes == 0 { return; }

                // Receive a generic Gateway Message
                let order_message: GatewayMessage = match serde_json::from_slice(&buf[..bytes]) {
                    Ok(order_message) => order_message,
                    Err(err) => panic!("Error {}", err)
                };

                // Handle and create a generic Gateway Message response
                let outbound_message = match order_message {
                    GatewayMessage::NewOrder(n) => Self::process_order(order_tx, n).await,
                    _ => panic!("Invalid Gateway Message!")
                };

                // Write response
                match serde_json::to_vec(&outbound_message) {
                    Ok(bytes) => tx.write_all(&*bytes).await.unwrap(),
                    Err(err) => panic!("Error {}", err)
                };
            }
            Err(_) => {}
        }
    }

    async fn process_order(order_tx: Sender<Order>, n: NewOrder) -> GatewayMessage {
        let side = match n.action {
            TradeAction::BUY => Side::BUY,
            TradeAction::SELL => Side::SELL
        };

        let engine_order = Order::new(random::<u32>(), n.qty, n.px, side);
        order_tx.send(engine_order).unwrap();

        return GatewayMessage::NewOrderAck(NewOrderAck {
            action: n.action,
            id: engine_order.identifier,
            px: n.px,
            qty: n.qty,
            ack_time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos(),
        });
    }
}