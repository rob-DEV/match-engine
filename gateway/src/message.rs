use bitcode::{Decode, Encode};
use common::domain::domain::{CancelOrder, NewOrder};

#[derive(Encode, Decode, PartialEq, Debug)]
pub enum GatewayMessage {
    LimitOrder(NewOrder),
    MarketOrder(NewOrder),
    CancelOrder(CancelOrder),
}
