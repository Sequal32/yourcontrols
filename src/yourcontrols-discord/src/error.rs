use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    // Secret decoder
    #[error("Invalid base64!")]
    Base64Error(#[from] base64::DecodeError),
    #[error("Invalid UTF8!")]
    UTF8Error(#[from] std::string::FromUtf8Error),
    #[error("Invalid SocketAddr!")]
    ParseError(#[from] std::net::AddrParseError),
}
