use common::types::order::OrderRequest;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::mpsc;

pub async fn market_gateway_sender(mut rx: mpsc::Receiver<OrderRequest>) {
    let mut socket = TcpStream::connect("127.0.0.1:3001")
        .await
        .expect("cannot connect to TCP market-gateway");

    println!("Connected to market-gateway.");

    while let Some(order) = rx.recv().await {
        let serialized = common::serialize::serialize::as_bytes(&order);

        if let Err(err) = socket.write_all(&serialized).await {
            println!("Gateway write error: {:?}", err);
            break;
        }
        if let Err(err) = socket.write_all(b"\n").await {
            println!("Gateway write error: {:?}", err);
            break;
        }
    }
}
