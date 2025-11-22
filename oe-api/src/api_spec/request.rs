use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub struct ApiOrderRequest {
    pub client_id: u32,
    pub instrument: String,
    pub side: String,
    pub px: u32,
    pub qty: u32,
    pub time_in_force: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub struct ApiOrderCancelRequest {
    pub client_id: u32,
    pub instrument: String,
    pub side: String,
    pub order_id: u32,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum IncomingMessage {
    ApiOrderRequest(ApiOrderRequest),
    ApiOrderCancelRequest(ApiOrderCancelRequest),
}
