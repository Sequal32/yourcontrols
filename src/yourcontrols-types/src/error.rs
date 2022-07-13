use std::{fmt::Display, io};

use crossbeam_channel::TryRecvError;

#[derive(Debug)]
pub enum Error {
    // Net
    IOError(io::Error),
    MismatchingIpVersion,

    SocketError(laminar::ErrorKind),
    GatewayNotFound(igd::SearchError),
    LocalAddrNotFound,
    LocalAddrNotIPv4(String),
    AddPortError(igd::AddPortError),

    ReadTimeout(TryRecvError),
    // Port forwarding

    // Definitions
    YamlError(serde_yaml::Error, String),
    MissingField(&'static str),
    InvalidSyncType(String),
    InvalidCategory(String),
    IncludeError(String, String),

    MissingMapping(String),
    // Serialization
    JSONSerializeError(serde_json::Error),
    NetDecodeError(rmp_serde::decode::Error),
    NetEncodeError(rmp_serde::encode::Error),

    // Discord
    Base64Error(base64::DecodeError),
    UTFError(std::string::FromUtf8Error),

    // Misc
    NotProcessed,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IOError(e) => write!(f, "An IO error occured: {}", e),
            Error::MismatchingIpVersion => {
                write!(f, "No hostname ips matched the requested IP version.")
            }
            Error::SocketError(e) => write!(f, "Could not initialize socket! Reason: {}", e),

            Error::GatewayNotFound(e) => write!(f, "Gateway not found: {}", e),
            Error::LocalAddrNotFound => write!(f, "Could not get local address."),
            Error::AddPortError(e) => write!(f, "Could not add port: {}", e),
            Error::LocalAddrNotIPv4(parse_string) => write!(f, "{} is not IPv4", parse_string),

            Error::MissingField(s) => write!(f, r#"Missing field "{}""#, s),
            Error::InvalidSyncType(s) => write!(f, r#"Invalid type "{}""#, s),
            Error::InvalidCategory(s) => write!(f, r#"Invalid category "{}""#, s),
            Error::YamlError(e, file_name) => {
                write!(f, "Error parsing YAML in {}: {}", file_name, e)
            }
            Error::IncludeError(e_str, e) => write!(f, "{} in {}", e_str, e),
            Error::MissingMapping(mapping_name) => write!(
                f,
                "No definition exists for {}. Do you have matching .yaml files?",
                mapping_name
            ),

            Error::JSONSerializeError(e) => {
                write!(f, "Could not serialize/deserialize! Reason: {}", e)
            }

            Error::ReadTimeout(_e) => write!(f, "No message."),
            Error::NetDecodeError(e) => {
                write!(f, "Could not decode MessagePack data! Reason: {}", e)
            }
            Error::NetEncodeError(e) => {
                write!(f, "Could not encode MessagePack data! Reason: {}", e)
            }
            Error::Base64Error(e) => write!(f, "Could not encode/decode base64! Reason: {}", e),
            Error::UTFError(e) => write!(f, "Could not convert UTF to string! Reason: {}", e),

            Error::NotProcessed => write!(f, "Not processed."),
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::IOError(e)
    }
}

impl From<laminar::ErrorKind> for Error {
    fn from(e: laminar::ErrorKind) -> Self {
        Error::SocketError(e)
    }
}

impl From<rmp_serde::decode::Error> for Error {
    fn from(e: rmp_serde::decode::Error) -> Self {
        Error::NetDecodeError(e)
    }
}

impl From<rmp_serde::encode::Error> for Error {
    fn from(e: rmp_serde::encode::Error) -> Self {
        Error::NetEncodeError(e)
    }
}

impl From<crossbeam_channel::TryRecvError> for Error {
    fn from(e: crossbeam_channel::TryRecvError) -> Self {
        Error::ReadTimeout(e)
    }
}

impl From<base64::DecodeError> for Error {
    fn from(e: base64::DecodeError) -> Self {
        Error::Base64Error(e)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Error::UTFError(e)
    }
}
