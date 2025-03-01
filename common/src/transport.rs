use crate::messaging::{Ack, CancelOrder, CancelOrderAck, EngineError, Logon, Logout, NewOrder, NewOrderAck, RejectionMessage, TradeExecution};
use bitcode::{Decode, Encode};

type SequenceNumber = u32;

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct InboundEngineMessage {
    pub sequence_number: SequenceNumber,
    pub message: EngineMessage
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct OutboundEngineMessage {
    pub sequence_number: SequenceNumber,
    pub message: EngineMessage
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub enum EngineMessage {
    NewOrder(NewOrder),
    NewOrderAck(NewOrderAck),
    CancelOrder(CancelOrder),
    CancelOrderAck(CancelOrderAck),
    TradeExecution(TradeExecution),

    NewOrderBatch([NewOrder; 256], u8),
    NewOrderAckBatch([NewOrderAck; 256], u8),
    CancelOrderBatch([CancelOrder; 256], u8),
    CancelOrderAckBatch([CancelOrderAck; 256], u8),
    TradeExecutionBatch([TradeExecution; 256], u8),

    RejectionMessage(RejectionMessage),
    EngineError(EngineError),
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub enum GatewayMessage {
    Logon(Logon),
    LogOut(Logout),
    NewOrder(NewOrder),
}