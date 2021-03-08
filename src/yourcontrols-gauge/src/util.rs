use std::fmt::Display;
/// The data size of the ClientDataArea.
pub const DATA_SIZE: usize = 8192;
/// A result acception any error.
pub type GenericResult<T> = Result<T, Box<dyn std::error::Error>>;
/// Type used for keeping track of vars mapped to values.
pub type DatumKey = u32;
/// Type used for getting/setting values.
pub type DatumValue = f64;
/// Time type used for interpolation.
pub type Time = f64;

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
