use common::types::cancel_order::CancelOrderRequest;
use common::types::order::OrderRequest;

pub enum GatewayMessage {
    LimitOrder(OrderRequest),
    MarketOrder(OrderRequest),
    CancelOrder(CancelOrderRequest),
}
