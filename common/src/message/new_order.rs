use crate::message::side::Side;
use bitcode::{Decode, Encode};

#[derive(Encode, Decode, PartialEq, Debug, Clone, Copy, Eq)]
pub struct NewOrderRequest {
    pub client_id: u32,
    pub order_side: Side,
    pub px: u32,
    pub qty: u32,
    pub time_in_force: TimeInForce,
    pub timestamp: u64,
}

#[derive(Encode, Decode, PartialEq, Debug, Clone, Copy, Eq)]
pub struct NewOrderAck {
    pub client_id: u32,
    pub side: Side,
    pub order_id: u32,
    pub px: u32,
    pub qty: u32,
    pub ack_time: u64,
}

#[derive(Encode, Decode, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeInForce {
    GTC,
    IOC,
    FOK,
}
