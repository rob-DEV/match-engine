use crate::types::side::Side::{Buy, Sell};

#[derive(PartialEq, Debug, Clone, Copy, Eq)]
#[repr(C)]
pub enum Side {
    Buy,
    Sell,
}

impl Side {
    pub fn str_to_type(side_str: &str) -> Side {
        match side_str.to_lowercase().as_str() {
            "buy" | "bid" => Buy,
            "sell" | "ask" => Sell,
            _ => panic!("Unknown order side"),
        }
    }
}
