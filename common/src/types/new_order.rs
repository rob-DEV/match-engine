use crate::types::side::Side;
use bitcode::{Decode, Encode};

#[derive(Encode, Decode, Debug, Clone, Copy)]
pub struct NewOrderRequest {
    pub client_id: u32,
    pub order_side: Side,
    pub px: u32,
    pub qty: u32,
    pub time_in_force: TimeInForce,
    pub timestamp: u64,
}

#[derive(Encode, Decode, Debug, Clone, Copy)]
pub struct NewOrderAck {
    pub client_id: u32,
    pub side: Side,
    pub order_id: u32,
    pub px: u32,
    pub qty: u32,
    pub ack_time: u64,
}

#[derive(Encode, Decode, Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimeInForce {
    GTC,
    IOC,
    FOK,
}
