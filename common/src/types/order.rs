use crate::types::side::Side;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct OrderRequest {
    pub client_id: u32,
    pub order_side: Side,
    pub px: u32,
    pub qty: u32,
    pub time_in_force: TimeInForce,
    pub timestamp: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NewOrderAck {
    pub client_id: u32,
    pub side: Side,
    pub order_id: u32,
    pub px: u32,
    pub qty: u32,
    pub ack_time: u64,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimeInForce {
    GTC,
    IOC,
    FOK,
}
