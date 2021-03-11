use std::fmt::Display;
/// The data size of the ClientDataArea.
pub const DATA_SIZE: usize = 8192;
/// A result acception any error.
pub type GenericResult<T> = Result<T, Box<dyn std::error::Error>>;

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
