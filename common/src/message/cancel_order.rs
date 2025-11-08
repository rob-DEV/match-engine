use crate::message::side::Side;
use bitcode::{Decode, Encode};

#[derive(Encode, Decode, PartialEq, Debug, Clone, Copy, Eq)]
pub struct CancelOrderRequest {
    pub client_id: u32,
    pub order_side: Side,
    pub order_id: u32,
}

#[derive(Encode, Decode, PartialEq, Debug, Clone, Copy, Eq)]
pub struct CancelledOrderAck {
    pub client_id: u32,
    pub order_id: u32,
    pub cancel_order_status: CancelOrderStatus,
    pub reason: Reason,
    pub ack_time: u64,
}

#[derive(Encode, Decode, PartialEq, Debug, Clone, Copy, Eq)]
pub enum CancelOrderStatus {
    NotFound,
    Cancelled,
}

#[derive(Encode, Decode, PartialEq, Debug, Clone, Copy, Eq)]
pub enum Reason {
    ClientRequested,
    SelfMatchPrevention,
}
