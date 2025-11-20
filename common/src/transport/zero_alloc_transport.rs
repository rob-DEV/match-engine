use crate::memory::memory::uninitialized_arr;
use crate::transport::sequenced_message::{
    SequencedEngineMessage, SequencedMessageRangeNack, MAX_UDP_MSG_BATCH_SIZE,
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

#[inline(always)]
pub fn as_bytes(msg: &RawWireMessage) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts(
            (msg as *const RawWireMessage) as *const u8,
            size_of::<u16>() + MAX_UDP_MSG_BATCH_SIZE * size_of::<SequencedEngineMessage>(),
        )
    }
}

#[inline(always)]
pub fn from_bytes(buf: &[u8]) -> &RawWireMessage {
    unsafe { &*(buf.as_ptr() as *const RawWireMessage) }
}

#[inline(always)]
pub fn nack_as_bytes(msg: &SequencedMessageRangeNack) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts(
            (msg as *const SequencedMessageRangeNack) as *const u8,
            size_of::<SequencedMessageRangeNack>(),
        )
    }
}

#[inline(always)]
pub fn nack_from_bytes(buf: &[u8]) -> &SequencedMessageRangeNack {
    unsafe { &*(buf.as_ptr() as *const SequencedMessageRangeNack) }
}
