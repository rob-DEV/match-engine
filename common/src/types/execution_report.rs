use std::fmt::Debug;

#[derive(PartialEq, Debug)]
#[repr(C)]
#[derive(Clone)]
pub struct ExecutionReport {
    pub trade_id: u32,

    pub bid_client_id: u32,
    pub bid_order_id: u32,
    pub bid_order_px: u32,
    pub bid_fill_type: FillType,

    pub ask_client_id: u32,
    pub ask_order_id: u32,
    pub ask_order_px: u32,
    pub ask_fill_type: FillType,

    pub instrument: [u8; 16],

    pub exec_px: u32,
    pub exec_qty: u32,
    pub exec_type: ExecType,

    pub exec_ns: u64,
}

#[derive(PartialEq, Debug, Copy, Clone)]
#[repr(C)]
pub enum ExecType {
    MatchEvent = 0,
    SelfMatchPrevented = 1,
}

#[derive(PartialEq, Debug, Copy, Clone)]
#[repr(C)]
pub enum FillType {
    NoFill = 0,
    PartialFill = 1,
    FullFill = 2,
}
