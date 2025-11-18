use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct L1 {
    pub best_bid: f64,
    pub best_ask: f64,
    pub last_price: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct L2Level {
    pub price: f64,
    pub qty: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct L2 {
    pub bids: [L2Level; 10],
    pub asks: [L2Level; 10],
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Trade {
    pub price: f64,
    pub qty: u64,
    pub ts: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MarketEvent {
    L1(L1),
    L2(L2),
    Trade(Trade),
}