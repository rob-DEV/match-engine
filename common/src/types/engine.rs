#[derive(PartialEq, Debug)]
#[repr(C)]
pub enum EngineCommand {
    Start,
    Shutdown,
}

#[derive(PartialEq, Debug)]
#[repr(C)]
pub enum EngineError {
    GeneralError,
}
