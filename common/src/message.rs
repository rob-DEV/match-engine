use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum GatewayMessage {
    // Market Data
    MarketDataRequest(MarketDataRequest),
    MarketDataResponse(MarketDataResponse),

    // Order
    NewOrder(NewOrder),
    NewOrderAck(NewOrderAck),
    NewOrderOnBookAck(NewOrderAck),
    CancelOrder(CancelOrder),
    CancelOrderAck(CancelOrderAck),

    // Match
    OrderFullyMatched(NewOrderAck),
    OrderPartiallyMatched(OrderPartiallyMatched),

    EngineError(EngineError),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SnapshotType {
    FullSnapshot,
    TopOfBook,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketDataRequest {
    pub snapshot_type: SnapshotType,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketDataEntry {
    pub px: u32,
    pub qty: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketDataTopOfBookSnapshot {
    pub snapshot_type: SnapshotType,
    pub bids: MarketDataEntry,
    pub asks: MarketDataEntry,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketDataFullSnapshot {
    pub snapshot_type: SnapshotType,
    pub bids: Vec<MarketDataEntry>,
    pub asks: Vec<MarketDataEntry>,
}

impl MarketDataFullSnapshot {
    pub fn new() -> MarketDataFullSnapshot {
        MarketDataFullSnapshot {
            snapshot_type: SnapshotType::FullSnapshot,
            bids: vec![],
            asks: vec![],
        }
    }
}


#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarketDataResponse {
    TopOfBook(MarketDataTopOfBookSnapshot),
    FullSnapshot(MarketDataFullSnapshot),
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
pub struct CancelOrder {
    pub action: TradeAction,
    pub order_id: u32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CancelOrderAck {
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineError {
    GeneralError,
}