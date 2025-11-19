use bitcode::{Decode, Encode};

#[derive(Encode, Decode, PartialEq, Debug)]
pub enum EngineCommand {
    Start,
    Shutdown,
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub enum EngineError {
    GeneralError,
}
