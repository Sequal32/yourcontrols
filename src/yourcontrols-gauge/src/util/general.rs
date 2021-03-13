
/// The data size of the ClientDataArea.
pub const DATA_SIZE: usize = 8192;
/// A result acception any error.
pub type GenericResult<T> = Result<T, Box<dyn std::error::Error>>;
