use crate::domain::domain::{CancelOrder, CancelOrderAck, EngineError, NewOrder, NewOrderAck, RejectionMessage, TradeExecution};
use bitcode::{Decode, Encode};

type SequenceNumber = u32;

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct InboundEngineMessage {
    pub sequence_number: SequenceNumber,
    pub message: EngineMessage,
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct OutboundEngineMessage {
    pub sequence_number: SequenceNumber,
    pub message: EngineMessage,
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