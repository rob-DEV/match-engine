use bitcode::{Decode, Encode};
use std::fmt::Debug;

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct ExecutionReport {
    pub trade_id: u32,
    pub trade_seq: u32,

    pub bid_client_id: u32,
    pub bid_order_id: u32,
    pub bid_order_px: u32,
    pub bid_fill_type: FillType,

    pub ask_client_id: u32,
    pub ask_order_id: u32,
    pub ask_order_px: u32,
    pub ask_fill_type: FillType,

    pub exec_px: u32,
    pub exec_qty: u32,
    pub exec_type: ExecType,

    pub exec_ns: u64,
}

#[derive(Encode, Decode, PartialEq, Debug, Copy, Clone)]
pub enum ExecType {
    MatchEvent,
    SelfMatchPrevented,
}

#[derive(Encode, Decode, PartialEq, Debug, Copy, Clone)]
pub enum FillType {
    NoFill,
    PartialFill,
    FullFill,
}
