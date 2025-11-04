use bitcode::{Decode, Encode};

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct RejectionMessage {
    pub reject_reason: u32,
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub enum EngineError {
    GeneralError,
}
