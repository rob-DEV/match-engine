use bitcode::{Decode, Encode};
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Encode, Decode, PartialEq, Debug, Clone, Copy, Eq)]
pub enum Side {
    BUY,
    SELL,
}
