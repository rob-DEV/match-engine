use crate::types::side::Side::{Buy, Sell};

#[derive(PartialEq, Debug, Clone, Copy, Eq)]
#[repr(C)]
pub enum Side {
    Buy = 0,
    Sell = 1,
}

impl Side {
    pub fn str_to_val(side_str: &str) -> Result<Side, String> {
        match side_str.to_lowercase().as_str() {
            "buy" | "bid" => Ok(Buy),
            "sell" | "ask" => Ok(Sell),
            _ => Err(format!("Unknown side {}", side_str)),
        }
    }

    pub fn val_to_str(side: Side) -> String {
        match side {
            Buy => "buy".to_owned(),
            Sell => "sell".to_owned(),
        }
    }
}
