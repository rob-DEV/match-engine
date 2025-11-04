use crate::message::side::Side;
use bitcode::{Decode, Encode};

#[derive(Encode, Decode, PartialEq, Debug, Clone, Copy, Eq)]
pub struct CancelOrder {
    pub client_id: u32,
    pub order_side: Side,
    pub order_id: u32,
}

#[derive(Encode, Decode, PartialEq, Debug, Clone, Copy, Eq)]
pub struct CancelOrderAck {
    pub client_id: u32,
    pub order_id: u32,
    pub found: bool,
    pub ack_time: u64,
}
