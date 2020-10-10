use serde::{Serialize, Deserialize};

#[derive(Eq, PartialEq)]
pub enum Category {
    Shared,
    Master,
    Server
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq)]
pub enum VarReaderTypes {
    Bool(bool),
    I32(i32),
    I64(i64),
    F64(f64)
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum InDataTypes {
    Bool,
    I32,
    I64,
    F64
}