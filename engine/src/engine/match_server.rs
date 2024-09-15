// use std::error::Error;
// use std::net::SocketAddr;
// use std::sync::{Arc, Mutex};
// use std::sync::mpsc::Sender;
//
// use rand::random;
// use tokio::io::{AsyncReadExt, AsyncWriteExt};
// use tokio::net::{TcpListener, TcpStream};
//
// use common::engine::{CancelOrder, CancelOrderAck, GatewayMessage, NewOrder, NewOrderAck};
// use common::engine::EngineError::GeneralError;
//
// use crate::domain::order::{LimitOrder, Order};
// use crate::util::time::epoch_nanos;
//
// pub struct MatchServer {
//     match_server_listener: TcpListener,
//     order_entry_tx_mutex: Arc<Mutex<Sender<Order>>>,
// }
//
// impl MatchServer {
//     pub async fn new(app_port: String, order_entry_tx: Sender<Order>) -> MatchServer {
//         println!("Starting Match Server on port {}", app_port);
//
//         let socket_addr = SocketAddr::from(([0, 0, 0, 0], app_port.parse().unwrap()));
//         MatchServer {
//             match_server_listener: TcpListener::bind(socket_addr).await.unwrap(),
//             order_entry_tx_mutex: Arc::new(Mutex::new(order_entry_tx)),
//         }
//     }
//
//     pub async fn run(&self) {
//         let order_entry_tx_mutex = self.order_entry_tx_mutex.clone();
//
//         loop {
//             let order_tx = order_entry_tx_mutex.lock().unwrap().clone();
//             let (socket, _) = self.match_server_listener.accept().await.unwrap();
//
//             tokio::spawn(async move {
//                 MatchServer::handler(socket, order_tx).await;
//             });
//         }
//     }
//
//     async fn handler(mut socket: TcpStream, new_order_tx: Sender<Order>) {
//         let (mut socket_rx, mut socket_tx) = socket.split();
//         let mut socket_rx_buffer: [u8; 4096] = [0; 4096];
//
//         match socket_rx.read(&mut socket_rx_buffer).await {
//             Ok(socket_rx_bytes_read) => {
//                 if socket_rx_bytes_read == 0 { return; }
//
//                 // Receive a generic Gateway Message
//                 let order_message: GatewayMessage = match serde_json::from_slice(&socket_rx_buffer[..socket_rx_bytes_read]) {
//                     Ok(order_message) => order_message,
//                     Err(err) => Self::handle_error(err)
//                 };
//
//                 // Handle and create a generic Gateway Message response
//                 let outbound_message = match order_message {
//                     GatewayMessage::NewOrder(new_order) => Self::handle_new_order(new_order_tx, new_order).await,
//                     GatewayMessage::CancelOrder(cancel_order) => Self::handle_cancel_order(new_order_tx, cancel_order).await,
//                     _ => GatewayMessage::EngineError(GeneralError)
//                 };
//
//                 // Write response
//                 match serde_json::to_vec(&outbound_message) {
//                     Ok(bytes) => socket_tx.write_all(&*bytes).await.unwrap(),
//                     Err(err) => Self::handle_error(err)
//                 };
//             }
//             Err(err) => Self::handle_error(err)
//         }
//     }
//
//     async fn handle_new_order(order_tx: Sender<Order>, new_order: NewOrder) -> GatewayMessage {
//         let limit_order = LimitOrder {
//             id: random::<u32>(),
//             action: new_order.action,
//             px: new_order.px,
//             qty: new_order.qty,
//             placed_time: 0,
//         };
//
//         order_tx.send(Order::New(limit_order)).unwrap();
//
//         return GatewayMessage::NewOrderAck(NewOrderAck {
//             id: limit_order.id,
//             action: new_order.action,
//             px: new_order.px,
//             qty: new_order.qty,
//             ack_time: epoch_nanos(),
//         });
//     }
//
//     async fn handle_cancel_order(order_tx: Sender<Order>, cancel_order: CancelOrder) -> GatewayMessage {
//         let cancel = crate::domain::order::CancelOrder {
//             id: cancel_order.id,
//             action: cancel_order.action,
//         };
//
//         order_tx.send(Order::Cancel(cancel)).unwrap();
//
//         return GatewayMessage::CancelOrderAck(CancelOrderAck {
//             ack_time: epoch_nanos(),
//         });
//     }
//
//     fn handle_error(err: impl Error) -> ! {
//         eprintln!("{:?}", err);
//         panic!()
//     }
// }