use serde::Deserialize;
use common::types::order::TimeInForce;


#[derive(Debug, Deserialize)]
pub struct ApiOrderRequest {
    pub client_id: u32,
    pub instrument: String,
    pub side: String,
    pub price: u32,
    pub qty: u32,
    pub time_in_force: String
}


#[derive(Debug, Deserialize)]
pub struct ApiOrderCancelRequest {
    pub client_id: u32,
    pub instrument: String,
    pub side: String,
    pub order_id: u32,
}
