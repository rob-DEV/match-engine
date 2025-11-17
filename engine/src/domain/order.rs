pub(crate) use crate::domain::limit_order::LimitOrder;
use common::types::cancel_order::CancelOrderRequest;

pub enum Order {
    LimitOrder(LimitOrder),
    Cancel(CancelOrderRequest),
}
