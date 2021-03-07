use std::fmt::Display;

pub const DATA_SIZE: usize = 8192;
pub type GenericResult<T> = Result<T, Box<dyn std::error::Error>>;
pub type DatumKey = u32;
pub type DatumValue = f64;
pub type Time = f64;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Message {
    fragment_index: u8,
    fragment_count: u8,
    bytes: Vec<u8>,
}

#[derive(Debug)]
pub enum Error {
    VariableInitializeError,
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
