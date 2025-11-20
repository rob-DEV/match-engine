use crate::types::cancel_order::{CancelOrderRequest, CancelledOrderAck};
use crate::types::engine::{EngineCommand, EngineError};
use crate::types::execution_report::ExecutionReport;
use crate::types::order::{NewOrderAck, OrderRequest};

pub const MAX_UDP_MSG_BATCH_SIZE: usize = 32;
pub type Subscriber = u32;
pub type SequenceNumber = u32;

#[repr(C)]
pub struct SequencedEngineMessage {
    pub sequence_number: SequenceNumber,
    pub message: EngineMessage,
    pub sent_time: u64,
}

#[repr(C)]
pub struct SequencedMessageRangeNack {
    pub start: SequenceNumber,
    pub end: SequenceNumber,
}

#[repr(C)]
#[derive(Debug)]
pub enum EngineMessage {
    // OE
    NewOrder(OrderRequest),
    NewOrderAck(NewOrderAck),
    CancelOrder(CancelOrderRequest),
    CancelOrderAck(CancelledOrderAck),
    TradeExecution(ExecutionReport),

    // SYS
    EngineCommand(EngineCommand),
    EngineError(EngineError),
}
