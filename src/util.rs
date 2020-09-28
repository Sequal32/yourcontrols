use serde::{Serialize, Deserialize};

pub struct LocalVar {
    pub category: Category,
    pub units: Option<String>,
    pub var_type: InDataTypes
}

pub enum Category {
    Shared,
    Master
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq)]
pub enum VarReaderTypes {
    Bool(bool),
    U32(u32),
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