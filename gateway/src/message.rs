use bitcode::{Decode, Encode};
use common::types::cancel_order::CancelOrderRequest;
use common::types::new_order::NewOrderRequest;

#[derive(Encode, Decode, Debug)]
pub enum GatewayMessage {
    LimitOrder(NewOrderRequest),
    MarketOrder(NewOrderRequest),
    CancelOrder(CancelOrderRequest),
}
