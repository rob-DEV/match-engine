use crate::message::cancel_order::{CancelOrderRequest, CancelledOrderAck};
use crate::message::engine::{EngineError, RejectionMessage};
use crate::message::execution_report::ExecutionReport;
use crate::message::new_order::{NewOrderAck, NewOrderRequest};
use bitcode::{Decode, Encode};

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
    RejectionMessage(RejectionMessage),

    // SYS
    EngineSubscriptionRequest(Subscriber),
    EngineSubscriptionPing(Subscriber),
    EngineSubscriptionEnd(Subscriber),
    EngineError(EngineError),
}
