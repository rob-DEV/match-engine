use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub struct ApiOrderAckResponse {
    pub client_id: u32,
    pub instrument: String,
    pub order_id: u32,
    pub side: String,
    pub px: u32,
    pub qty: u32,
    pub ack_time: u64,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub struct ApiCancelOrderAckResponse {
    pub client_id: u32,
    pub instrument: String,
    pub order_id: u32,
    pub cancel_order_status: String,
    pub reason: String,
    pub ack_time: u64,
}
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub struct ApiExecutionReportResponse {
    pub client_id: u32,
    pub instrument: String,
    pub order_id: u32,
    pub fill_type: String,
    pub exec_px: u32,
    pub exec_qty: u32,
    pub exec_type: String,
    pub exec_ns: u64,
}
