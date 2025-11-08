use bitcode::{Decode, Encode};
use common::message::cancel_order::CancelOrderRequest;
use common::message::new_order::NewOrderRequest;

#[derive(Encode, Decode, Debug)]
pub enum GatewayMessage {
    LimitOrder(NewOrderRequest),
    MarketOrder(NewOrderRequest),
    CancelOrder(CancelOrderRequest),
}
