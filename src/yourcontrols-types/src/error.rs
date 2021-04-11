use std::{fmt::Display, io};

use crossbeam_channel::TryRecvError;

pub type Result<T> = std::result::Result<T, Error>;

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

    // Crossbeam
    ReadTimeout(TryRecvError),
    // Port forwarding

    // Definitions
    YamlError(serde_yaml::Error, String),
    YamlError2(serde_yaml::Error),
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

    // Websocket
    WebsocketError(tungstenite::Error),
    // Gauge
    VariableInitializeError,
    // Gauge Scripting
    RhaiParse(rhai::ParseError),
    RhaiError(Box<rhai::EvalAltResult>),

    // Misc
    Base64DecodeError(base64::DecodeError),
    ConvertToStringError(std::string::FromUtf8Error),
    AddressParseError(std::net::AddrParseError),
    None,
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
                write!(f, "Error parsing YAML in {}: {}", file_name, e.to_string())
            }
            Error::YamlError2(e) => {
                write!(f, "Error parsing YAML: {}", e.to_string())
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
            Error::WebsocketError(e) => {
                write!(f, "Could not send message through websocket: {:?}", e)
            }

            Error::VariableInitializeError => write!(f, "Var could not be initialized."),
            Error::RhaiParse(e) => write!(f, "Could not parse RHAI script: {}", e),
            Error::RhaiError(e) => write!(f, "Could not run RHAI script: {}", e),

            Error::None => write!(f, "No value returned."),
            Error::NotProcessed => write!(f, "Not processed."),
            Error::Base64DecodeError(e) => write!(f, "Could not decode data from base64: {}", e),
            Error::ConvertToStringError(e) => write!(f, "Could not convert bytes to string: {}", e),
            Error::AddressParseError(e) => write!(f, "Could not parse socket address: {}", e),
        }
    }
}

macro_rules! from_err {
    ($from_error: ty, $to_variant: ident) => {
        impl From<$from_error> for Error {
            fn from(e: $from_error) -> Self {
                Error::$to_variant(e)
            }
        }
    };
}

from_err!(io::Error, IOError);
from_err!(laminar::ErrorKind, SocketError);
from_err!(rmp_serde::decode::Error, NetDecodeError);
from_err!(rmp_serde::encode::Error, NetEncodeError);
from_err!(crossbeam_channel::TryRecvError, ReadTimeout);
from_err!(Box<rhai::EvalAltResult>, RhaiError);
from_err!(serde_yaml::Error, YamlError2);
from_err!(base64::DecodeError, Base64DecodeError);
from_err!(tungstenite::Error, WebsocketError);
from_err!(std::string::FromUtf8Error, ConvertToStringError);
from_err!(std::net::AddrParseError, AddressParseError);
