use crate::types::cancel_order::{CancelOrderRequest, CancelledOrderAck};
use crate::types::engine::{EngineCommand, EngineError};
use crate::types::execution_report::ExecutionReport;
use crate::types::new_order::{NewOrderAck, NewOrderRequest};
use bitcode::{Decode, Encode};

pub const MAX_UDP_MSG_BATCH_SIZE: usize = 32;
pub type Subscriber = u32;
pub type SequenceNumber = u32;

#[derive(Encode, Decode, Debug)]
pub struct SequencedEngineMessage {
    pub sequence_number: SequenceNumber,
    pub message: EngineMessage,
    pub sent_time: u64,
}

#[derive(Encode, Decode, Debug)]
pub struct SequencedMessageAck {
    pub subscriber: Subscriber,
    pub sequence_number: SequenceNumber,
}

#[derive(Encode, Decode, Debug)]
pub struct SequencedMessageNack {
    pub requested_sequence_number: SequenceNumber,
}

#[derive(Encode, Decode, Debug)]
pub struct SequencedMessageRangeNack {
    pub start: SequenceNumber,
    pub end: SequenceNumber,
}

#[derive(Encode, Decode, Debug)]
pub enum EngineMessage {
    // OE
    NewOrder(NewOrderRequest),
    NewOrderAck(NewOrderAck),
    CancelOrder(CancelOrderRequest),
    CancelOrderAck(CancelledOrderAck),
    TradeExecution(ExecutionReport),

    // SYS
    EngineCommand(EngineCommand),
    EngineError(EngineError),
}
