use std::error::Error;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use std::time::{SystemTime, UNIX_EPOCH};

use rand::random;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use common::domain::{CancelOrder, CancelOrderAck, GatewayMessage, MarketDataFullSnapshot, MarketDataRequest, MarketDataTopOfBookSnapshot, NewOrder, NewOrderAck, SnapshotType};
use common::domain::EngineError::GeneralError;
use common::domain::GatewayMessage::MarketDataResponse;
use common::domain::MarketDataResponse::{FullSnapshot, TopOfBook};

use crate::domain::order::{LimitOrder, Order};

pub struct MatchServer {
    match_server_listener: TcpListener,
    order_entry_tx_mutex: Arc<Mutex<Sender<Order>>>,
    market_data_rx_mutex: Arc<Mutex<MarketDataFullSnapshot>>,
}

impl MatchServer {
    pub async fn new(app_port: String, order_entry_tx: Sender<Order>, market_data_rx: Arc<Mutex<MarketDataFullSnapshot>>) -> MatchServer {
        println!("Starting Match Server on port {}", app_port);

        let socket_addr = SocketAddr::new("127.0.0.1".parse().unwrap(), app_port.parse().unwrap());
        MatchServer {
            match_server_listener: TcpListener::bind(socket_addr).await.unwrap(),
            order_entry_tx_mutex: Arc::new(Mutex::new(order_entry_tx)),
            market_data_rx_mutex: market_data_rx,
        }
    }

    pub async fn run(&self) {
        let order_entry_tx_mutex = self.order_entry_tx_mutex.clone();

        loop {
            let order_tx = order_entry_tx_mutex.lock().unwrap().clone();
            let market_data_rx = self.market_data_rx_mutex.clone();
            let (socket, _) = self.match_server_listener.accept().await.unwrap();

            tokio::spawn(async move {
                MatchServer::handler(socket, order_tx, market_data_rx).await;
            });
        }
    }

    async fn handler(mut socket: TcpStream, new_order_tx: Sender<Order>, market_data_rx_mutex: Arc<Mutex<MarketDataFullSnapshot>>) {
        let (mut socket_rx, mut socket_tx) = socket.split();
        let mut socket_rx_buffer: [u8; 512] = [0; 512];

        match socket_rx.read(&mut socket_rx_buffer).await {
            Ok(socket_rx_bytes_read) => {
                if socket_rx_bytes_read == 0 { return; }

                // Receive a generic Gateway Message
                let order_message: GatewayMessage = match serde_json::from_slice(&socket_rx_buffer[..socket_rx_bytes_read]) {
                    Ok(order_message) => order_message,
                    Err(err) => Self::handle_error(err)
                };

                // Handle and create a generic Gateway Message response
                let outbound_message = match order_message {
                    GatewayMessage::NewOrder(new_order) => Self::handle_new_order(new_order_tx, new_order).await,
                    GatewayMessage::CancelOrder(cancel_order) => Self::handle_cancel_order(new_order_tx, cancel_order).await,
                    GatewayMessage::MarketDataRequest(market_data_request) => Self::handle_market_data_request(market_data_rx_mutex, market_data_request).await,
                    _ => GatewayMessage::EngineError(GeneralError)
                };

                // Write response
                match serde_json::to_vec(&outbound_message) {
                    Ok(bytes) => socket_tx.write_all(&*bytes).await.unwrap(),
                    Err(err) => Self::handle_error(err)
                };
            }
            Err(err) => Self::handle_error(err)
        }
    }

    async fn handle_new_order(order_tx: Sender<Order>, new_order: NewOrder) -> GatewayMessage {
        let limit_order = LimitOrder {
            id: random::<u32>(),
            px: new_order.px,
            qty: new_order.qty,
            side: new_order.action,
            placed_time: 0,
        };

        order_tx.send(Order::New(limit_order)).unwrap();

        return GatewayMessage::NewOrderAck(NewOrderAck {
            action: new_order.action,
            id: limit_order.id,
            px: new_order.px,
            qty: new_order.qty,
            ack_time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos(),
        });
    }

    async fn handle_cancel_order(order_tx: Sender<Order>, cancel_order: CancelOrder) -> GatewayMessage {
        let cancel = CancelOrder {
            action: cancel_order.action,
            id: cancel_order.id,
        };

        order_tx.send(Order::Cancel(cancel)).unwrap();

        return GatewayMessage::CancelOrderAck(CancelOrderAck {
            ack_time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos(),
        });
    }

    async fn handle_market_data_request(market_data_rx_mutex: Arc<Mutex<MarketDataFullSnapshot>>, market_data_request: MarketDataRequest) -> GatewayMessage {
        let snapshot = market_data_rx_mutex.lock().unwrap();
        match market_data_request.snapshot_type {
            SnapshotType::FullSnapshot => MarketDataResponse(FullSnapshot(MarketDataFullSnapshot {
                snapshot_type: SnapshotType::FullSnapshot,
                bids: snapshot.bids.clone(),
                asks: snapshot.asks.clone(),
            })),
            SnapshotType::TopOfBook => MarketDataResponse(TopOfBook(MarketDataTopOfBookSnapshot {
                snapshot_type: SnapshotType::TopOfBook,
                bids: snapshot.bids.first().unwrap().clone(),
                asks: snapshot.asks.first().unwrap().clone(),
            })),
        }
    }

    fn handle_error(err: impl Error) -> ! {
        eprintln!("{:?}", err);
        panic!()
    }
}