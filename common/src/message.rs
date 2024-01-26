use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum GatewayMessage {
    NewOrder(NewOrder),
    NewOrderAck(NewOrderAck),
    NewOrderOnBookAck(NewOrderAck),
    OrderFullyMatched(NewOrderAck),
    OrderPartiallyMatched(OrderPartiallyMatched),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TradeAction {
    BUY,
    SELL,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewOrder {
    pub action: TradeAction,
    pub px: u32,
    pub qty: u32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewOrderAck {
    pub action: TradeAction,
    pub id: u32,
    pub px: u32,
    pub qty: u32,
    pub ack_time: u128,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderFullyMatched {
    pub id: u32,
    pub order_id: u32,
    pub px: u32,
    pub qty: u32,
    pub execution_time: u128,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderPartiallyMatched {
    pub id: u32,
    pub order_id: u32,
    pub px: u32,
    pub qty: u32,
    pub qty_remaining: u32,
    pub execution_time: u128,
}