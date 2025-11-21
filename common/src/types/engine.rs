#[derive(PartialEq, Debug)]
#[repr(C)]
#[derive(Clone)]
pub enum EngineCommand {
    Start,
    Shutdown,
}

#[derive(PartialEq, Debug)]
#[repr(C)]
#[derive(Clone)]
pub enum EngineError {
    GeneralError,
}
