pub(crate) use crate::domain::limit_order::LimitOrder;
use common::message::cancel_order::CancelOrderRequest;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Order {
    LimitOrder(LimitOrder),
    Cancel(CancelOrderRequest),
}
