use crate::memory::memory::uninitialized_arr;
use crate::transport::sequenced_message::{
    SequencedEngineMessage, MAX_UDP_MSG_BATCH_SIZE,
};

#[repr(C)]
pub struct RawWireMessage {
    pub batch_size: u16,
    pub batch: [SequencedEngineMessage; MAX_UDP_MSG_BATCH_SIZE],
}

impl RawWireMessage {
    pub fn default() -> Self {
        RawWireMessage {
            batch_size: 0,
            batch: uninitialized_arr(),
        }
    }
}