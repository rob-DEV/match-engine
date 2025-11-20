use crate::types::side::Side;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct CancelOrderRequest {
    pub client_id: u32,
    pub order_side: Side,
    pub order_id: u32,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct CancelledOrderAck {
    pub client_id: u32,
    pub order_id: u32,
    pub cancel_order_status: CancelOrderStatus,
    pub reason: Reason,
    pub ack_time: u64,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub enum CancelOrderStatus {
    NotFound,
    Cancelled,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub enum Reason {
    ClientRequested,
    SelfMatchPrevention,
}
