pub(crate) use crate::domain::limit_order::LimitOrder;
use common::message::cancel_order::CancelOrderRequest;

#[derive(Copy, Clone, Debug)]
pub enum Order {
    LimitOrder(LimitOrder),
    Cancel(CancelOrderRequest),
}
