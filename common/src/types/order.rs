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
    pub order_id: u32,
    pub instrument: [u8; 16],
    pub side: Side,
    pub px: u32,
    pub qty: u32,
    pub qty_rem: u32,
    pub time_in_force: TimeInForce,
    pub ack_time: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimeInForce {
    GTC = 0,
    IOC = 1,
    FOK = 2,
}

impl TimeInForce {
    pub fn str_to_val(time_in_force: &str) -> Result<TimeInForce, String> {
        match time_in_force.to_lowercase().as_str() {
            "gtc" => Ok(TimeInForce::GTC),
            "ioc" => Ok(TimeInForce::IOC),
            "fok" => Ok(TimeInForce::FOK),
            _ => Err(format!("Invalid TimeInForce: {}", time_in_force)),
        }
    }
}
