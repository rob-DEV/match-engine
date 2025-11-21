use crate::types::side::Side;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct OrderRequest {
    pub client_id: u32,
    pub instrument: [u8; 16],
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

impl TimeInForce {
    pub fn str_to_type(time_in_force: &str) -> TimeInForce {
        match time_in_force {
            "GTC" => TimeInForce::GTC,
            "IOC" => TimeInForce::IOC,
            "FOK" => TimeInForce::FOK,
            _ => panic!("Unknown time_in_force: {}", time_in_force),
        }
    }
}
