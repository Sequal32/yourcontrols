use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error occured with the Laminar socket!")]
    SocketError(#[from] laminar::ErrorKind),

    #[cfg(not(debug_assertions))]
    #[error("Error encoding packet!")]
    SerializeError(#[from] rmp_serde::encode::Error),
    #[cfg(not(debug_assertions))]
    #[error("Error decoding packet!")]
    DeserializeError(#[from] rmp_serde::decode::Error),

    #[cfg(debug_assertions)]
    #[error("Error encoding packet!")]
    SerializeError(#[from] serde_json::Error),
    // Decoding IP
    #[error("Invalid base64!")]
    Base64Error(#[from] base64::DecodeError),
    #[error("Invalid UTF8!")]
    UTF8Error(#[from] std::string::FromUtf8Error),
    #[error("Invalid SocketAddr!")]
    ParseError(#[from] std::net::AddrParseError),
}
