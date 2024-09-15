use crate::domain::order::Order;
use std::env;
use std::error::Error;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::mpsc;
use common::engine::InboundEngineMessage;

mod engine;
mod domain;
mod util;


fn main() -> Result<(), Box<dyn Error>> {
    println!("Initializing Engine on port:{}", 3000);

    // Engine Channels - Order Entry & Market Data
    let (order_entry_tx, order_entry_rx): (mpsc::Sender<Order>, mpsc::Receiver<Order>) = mpsc::channel();

    // Engine started on separate non-tokio threads
    // let match_engine = engine::match_engine::MatchEngine::new();
    // match_engine.run(order_entry_rx);

    // Order entry tokio rt
    let app_port = env::var("APP_PORT").unwrap_or("3000".to_string());

    // let match_server = engine::match_server::MatchServer::new(app_port, order_entry_tx);
    // match_server.await.run().await;

    use socket2::{Domain, Protocol, Socket, Type};
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    socket.set_reuse_address(true)?;
    socket.set_reuse_port(true)?;
    socket.bind(&SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 3000).into())?;

    let udp_socket = std::net::UdpSocket::from(socket);
    println!("Listening on: {}", udp_socket.local_addr()?);

    let mut buffer = [0; 1024];
    let mut req_per_second: usize = 0;
    let mut time = minstant::Instant::now();

    loop {
        let (size, addr) = udp_socket.recv_from(&mut buffer).unwrap();
        let inbound_engine_message: InboundEngineMessage = bitcode::decode(&buffer[..size]).unwrap();

        if time.elapsed().as_millis() > 1000 {
            time = minstant::Instant::now();
            println!("Req / sec: {}", req_per_second);
            req_per_second = 0;
        }

        req_per_second += 1;
    }

    Ok(())
}