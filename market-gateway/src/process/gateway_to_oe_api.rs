use common::transport::sequenced_message::EngineMessage;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::broadcast::Receiver;

pub async fn gateway_to_oe_api_handler(
    mut tx_oe_api_stream: OwnedWriteHalf,
    mut rx_engine_queue: Receiver<EngineMessage>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Initialized GW -> OE_API");
    while let Ok(message) = rx_engine_queue.recv().await {
        let serialized = common::serialize::serialize::as_bytes(&message);
        let packet_len = serialized.len();
        if let Err(err) = tx_oe_api_stream.write_all(&packet_len.to_be_bytes()).await {
            println!("Gateway write error: {:?}", err);
        }

        if let Err(err) = tx_oe_api_stream.write_all(&serialized).await {
            println!("Gateway write error: {:?}", err);
            break;
        }
    }

    Ok(())
}
