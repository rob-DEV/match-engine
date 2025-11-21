use crate::app_state::AppState;
use std::mem::MaybeUninit;
use std::ptr;

use common::transport::sequenced_message::EngineMessage;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::net::tcp::OwnedReadHalf;

pub async fn oe_api_to_gateway_handler(
    mut stream: OwnedReadHalf,
    state: Arc<AppState>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Initialized OE_API -> GW");

    let mut buffer = [0u8; 4096];

    loop {
        let mut len_buf = [0u8; 4];
        if stream.read_exact(&mut len_buf).await.is_err() {
            return Ok(()); // disconnected
        }

        let frame_len = u32::from_be_bytes(len_buf) as usize;

        if frame_len > 0 {
            if frame_len > buffer.len() {
                println!("Frame too large");
                return Ok(());
            }

            stream.read_exact(&mut buffer[..frame_len]).await?;

            let mut msg = MaybeUninit::<EngineMessage>::uninit();

            // Own message TODO: fix serialization code
            unsafe {
                ptr::copy_nonoverlapping(buffer.as_ptr(), msg.as_mut_ptr() as *mut u8, frame_len);
                let msg = msg.assume_init();
                state.tx_oe_queue.send(msg).await?;
            }
        }
    }
}
