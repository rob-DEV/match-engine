use bitcode::{Decode, Encode};

#[derive(Encode, Decode, PartialEq, Debug, Clone, Copy, Eq)]
pub enum Side {
    BUY,
    SELL,
}
