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

    // Error
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
            bids: Vec::with_capacity(100),
            asks: Vec::with_capacity(100),
        }
    }
}


#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarketDataResponse {
    TopOfBook(MarketDataTopOfBookSnapshot),
    FullSnapshot(MarketDataFullSnapshot),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderAction {
    BUY,
    SELL,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewOrder {
    pub action: OrderAction,
    pub px: u32,
    pub qty: u32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CancelOrder {
    pub action: OrderAction,
    pub id: u32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewOrderAck {
    pub action: OrderAction,
    pub id: u32,
    pub px: u32,
    pub qty: u32,
    pub ack_time: u64,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CancelOrderAck {
    pub ack_time: u64,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineError {
    GeneralError,
}