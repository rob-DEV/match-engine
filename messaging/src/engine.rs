use bitcode::{Decode, Encode};
#[derive(Encode, Decode, PartialEq, Debug)]
pub struct InboundEngineMessage {
    pub seq_num: u32,
    pub inbound_message: InboundMessage,
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct OutboundEngineMessage {
    pub session_id: u32,
    pub seq_num: u32,
    pub outbound_message: OutboundMessage,
}


#[derive(Encode, Decode, PartialEq, Debug)]
pub enum InboundMessage {
    NewOrder(NewOrder),
    CancelOrder(CancelOrder),
}


#[derive(Encode, Decode, PartialEq, Debug)]
pub enum OutboundMessage {
    NewOrderAck(NewOrderAck),
    CancelOrderAck(CancelOrderAck),
    RejectionMessage(RejectionMessage),

    TradeExecution(TradeExecution),


    EngineError(EngineError),
}

#[derive(Encode, Decode, PartialEq, Debug, Clone, Copy, Eq)]
pub enum OrderAction {
    BUY,
    SELL,
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct NewOrder {
    pub order_action: OrderAction,
    pub px: u32,
    pub qty: u32,
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct CancelOrder {
    pub order_action: OrderAction,
    pub order_id: u32,
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct NewOrderAck {
    pub action: OrderAction,
    pub order_id: u32,
    pub px: u32,
    pub qty: u32,
    pub ack_time: u64,
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct CancelOrderAck {
    pub order_id: u32,
    pub ack_time: u64,
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct TradeExecution {
    pub execution_id: u32
}


#[derive(Encode, Decode, PartialEq, Debug)]
pub struct RejectionMessage {
    pub reject_reason: u32
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub enum EngineError {
    GeneralError,
}