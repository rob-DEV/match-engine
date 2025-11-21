use common::transport::sequenced_message::EngineMessage;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::mpsc;

pub async fn gateway_order_entry(
    mut tx_gateway_stream: OwnedWriteHalf,
    mut rx: mpsc::Receiver<EngineMessage>,
) {
    println!("Initialized OE-API -> GW");

    while let Some(msg_for_engine) = rx.recv().await {
        let serialized = common::serialize::serialize::as_bytes(&msg_for_engine);

        let packet_len = serialized.len();
        if let Err(err) = tx_gateway_stream.write_all(&packet_len.to_be_bytes()).await {
            println!("Gateway write error: {:?}", err);
        }

        if let Err(err) = tx_gateway_stream.write_all(&serialized).await {
            println!("Gateway write error: {:?}", err);
            break;
        }
    }
}
