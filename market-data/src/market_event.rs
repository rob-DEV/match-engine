use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct L1 {
    pub best_bid: u32,
    pub best_ask: u32,
    pub last_price: u32,
}

impl L1 {
    pub fn default() -> L1 {
        L1 {
            best_bid: 0,
            best_ask: 0,
            last_price: 0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct L2Level {
    pub px: u32,
    pub qty: u32,
}

impl L2Level {
    pub fn default() -> L2Level {
        L2Level { px: 0, qty: 0 }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct L2 {
    pub bids: [L2Level; 10],
    pub asks: [L2Level; 10],
}

impl L2 {
    pub fn default() -> L2 {
        L2 {
            bids: [L2Level::default(); 10],
            asks: [L2Level::default(); 10],
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Trade {
    pub px: u32,
    pub qty: u32,
    pub ts: u64,
}

impl Trade {
    pub fn default() -> Trade {
        Trade {
            px: 0,
            qty: 0,
            ts: 0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct MarketEvent {
    pub l1: L1,
    pub l2: L2, // full snapshot or diff
    pub last_px: u32,
    pub trades: [Trade; 10],
}
