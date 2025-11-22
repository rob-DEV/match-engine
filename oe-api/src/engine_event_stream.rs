use crate::app_state::AppState;
use common::transport::sequenced_message::EngineMessage;
use std::mem::MaybeUninit;
use std::ptr;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::net::tcp::OwnedReadHalf;

pub async fn gateway_event_stream(mut rx_gateway_stream: OwnedReadHalf, mut state: Arc<AppState>) {
    println!("Initialized GW -> OE-API");

    let mut buffer = [0u8; 4096];

    loop {
        let mut len_buf = [0u8; 4];
        if rx_gateway_stream.read_exact(&mut len_buf).await.is_err() {
            return; // disconnected
        }

        let frame_len = u32::from_be_bytes(len_buf) as usize;

        if frame_len > 0 {
            if frame_len > buffer.len() {
                println!("Frame too large");
                return;
            }

            rx_gateway_stream
                .read_exact(&mut buffer[..frame_len])
                .await
                .unwrap();

            let mut msg = MaybeUninit::<EngineMessage>::uninit();

            // Own message TODO: fix serialization code
            unsafe {
                ptr::copy_nonoverlapping(buffer.as_ptr(), msg.as_mut_ptr() as *mut u8, frame_len);
                let msg = msg.assume_init();

                let (client_id, other_client) = match &msg {
                    EngineMessage::NewOrder(new_order) => (new_order.client_id, 0),
                    EngineMessage::NewOrderAck(new_order_ack) => (new_order_ack.client_id, 0),
                    EngineMessage::CancelOrder(cancel_order) => (cancel_order.client_id, 0),
                    EngineMessage::CancelOrderAck(cancel_order_ack) => {
                        (cancel_order_ack.client_id, 0)
                    }
                    EngineMessage::TradeExecution(execution) => {
                        (execution.bid_client_id, execution.ask_client_id)
                    }

                    EngineMessage::EngineCommand(_) => {
                        panic!("Received Engine Command")
                    }
                    EngineMessage::EngineError(_) => {
                        panic!("Received Engine Error")
                    }
                };

                if let Some(client_channel) = state.tx_engine_to_client_channel.get(&other_client) {
                    let clone = msg.clone();
                    client_channel.send(clone).await.unwrap();
                }

                if let Some(channel) = state.tx_engine_to_client_channel.get(&client_id) {
                    channel.send(msg).await.unwrap();
                }
            }
        }
    }
}
