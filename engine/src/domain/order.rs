use common::message::cancel_order::CancelOrder;
pub(crate) use crate::domain::limit_order::LimitOrder;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Order {
    LimitOrder(LimitOrder),
    Cancel(CancelOrder),
}
