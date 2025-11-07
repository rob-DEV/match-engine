use crate::message::cancel_order::{CancelOrder, CancelOrderAck};
use crate::message::engine::{EngineError, RejectionMessage};
use crate::message::execution::TradeExecution;
use crate::message::new_order::{NewOrder, NewOrderAck};
use bitcode::{Decode, Encode};

pub type Subscriber = u32;
pub type SequenceNumber = u32;

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct SequencedEngineMessage {
    pub sequence_number: SequenceNumber,
    pub message: EngineMessage,
    pub sent_time: u64,
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct SequencedMessageAck {
    pub subscriber: Subscriber,
    pub sequence_number: SequenceNumber,
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub enum EngineMessage {
    // OE
    NewOrder(NewOrder),
    NewOrderAck(NewOrderAck),
    CancelOrder(CancelOrder),
    CancelOrderAck(CancelOrderAck),
    TradeExecution(TradeExecution),
    RejectionMessage(RejectionMessage),

    // SYS
    EngineSubscriptionRequest(Subscriber),
    EngineSubscriptionPing(Subscriber),
    EngineSubscriptionEnd(Subscriber),
    EngineError(EngineError),
}
