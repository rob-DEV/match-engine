use crate::domain::domain::{CancelOrder, CancelOrderAck, EngineError, NewOrder, NewOrderAck, RejectionMessage, TradeExecution};
use bitcode::{Decode, Encode};

pub type SequenceNumber = u32;

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct SequencedEngineMessage {
    pub sequence_number: SequenceNumber,
    pub message: EngineMessage,
    pub sent_time: u64
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub enum EngineMessage {
    NewOrder(NewOrder),
    NewOrderAck(NewOrderAck),
    CancelOrder(CancelOrder),
    CancelOrderAck(CancelOrderAck),
    TradeExecution(TradeExecution),
    RejectionMessage(RejectionMessage),
    EngineError(EngineError),
}