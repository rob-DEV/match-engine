use bitcode::{Decode, Encode};

#[derive(Encode, Decode, PartialEq, Debug, Clone, Copy, Eq)]
pub enum Side {
    BUY,
    SELL,
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct NewOrder {
    pub client_id: u32,
    pub order_action: Side,
    pub px: u32,
    pub qty: u32,
    pub timestamp: u64
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct CancelOrder {
    pub client_id: u32,
    pub order_action: Side,
    pub order_id: u32,
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct NewOrderAck {
    pub client_id: u32,
    pub action: Side,
    pub order_id: u32,
    pub px: u32,
    pub qty: u32,
    pub ack_time: u64,
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct CancelOrderAck {
    pub client_id: u32,
    pub order_id: u32,
    pub found: bool,
    pub ack_time: u64,
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct RejectionMessage {
    pub reject_reason: u32,
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct TradeExecution {
    pub trade_id: u32,
    pub trade_seq: u32,
    pub bid_client_id: u32,
    pub ask_client_id: u32,
    pub bid_order_id: u32,
    pub ask_order_id: u32,
    pub fill_qty: u32,
    pub px: u32,
    pub execution_time: u64,
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub enum EngineError {
    GeneralError,
}