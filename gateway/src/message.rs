use bitcode::{Decode, Encode};
use common::message::cancel_order::CancelOrder;
use common::message::new_order::NewOrder;

#[derive(Encode, Decode, PartialEq, Debug)]
pub enum GatewayMessage {
    LimitOrder(NewOrder),
    MarketOrder(NewOrder),
    CancelOrder(CancelOrder),
}
