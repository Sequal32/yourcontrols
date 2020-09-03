use std::io::{Read, Cursor};
use indexmap::IndexMap;
use serde::{Serialize, Deserialize};
#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub enum StructDataTypes {
    Bool(bool),
    I32(i32),
    I64(i64),
    F64(f64)
}
#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub enum InDataTypes {
    Bool,
    I32,
    I64,
    F64
}

#[derive(Debug)]
pub struct StructData {
    data_map: IndexMap<String, InDataTypes>,
}

impl StructData {
    pub fn new() -> Self {
        Self {
            data_map: IndexMap::new()
        }
    }

    pub fn add_definition(&mut self, key: String, data_type: InDataTypes) {
        self.data_map.insert(key, data_type);
    }

    pub fn read_from_bytes(&self, data: Box<[u8]>) -> IndexMap<String, StructDataTypes> {
        let mut return_data: IndexMap<String, StructDataTypes> = IndexMap::new();
        let mut reader = Cursor::new(data);
        for (key, data_type) in &self.data_map {
            unsafe {
                match data_type {
                    InDataTypes::Bool => {
                        let mut buf: [u8; 8] = [0; 8];
                        reader.read_exact(&mut buf).ok();
                        return_data.insert(key.to_string(), StructDataTypes::Bool(std::mem::transmute_copy(&buf)))
                    }
                    InDataTypes::I32 => {
                        let mut buf: [u8; 8] = [0; 8];
                        reader.read_exact(&mut buf).ok();
                        return_data.insert(key.to_string(), StructDataTypes::I32(std::mem::transmute_copy(&buf)))
                    }
                    InDataTypes::I64 => {
                        let mut buf: [u8; 8] = [0; 8];
                        reader.read_exact(&mut buf).ok();
                        return_data.insert(key.to_string(), StructDataTypes::I64(std::mem::transmute_copy(&buf)))
                    }
                    InDataTypes::F64 => {
                        let mut buf: [u8; 8] = [0; 8];
                        reader.read_exact(&mut buf).ok();
                        return_data.insert(key.to_string(), StructDataTypes::F64(std::mem::transmute_copy(&buf)))
                    }
                };
            }
        }
        return return_data;
    }

    pub fn read_from_dword(&self, data: [u8; 1024]) -> IndexMap<String, StructDataTypes> {
        self.read_from_bytes(Box::new(data))
    }

    pub fn write_to_data(&self, data: &IndexMap<String, StructDataTypes>) -> Vec<u8> {
        let mut buf: Vec<u8> = vec![];
        for (name, _) in &self.data_map {
            let val = data.get(name).unwrap();
            match val {
                StructDataTypes::Bool(n) => buf.extend((*n as i64).to_le_bytes().iter()),
                StructDataTypes::I32(n) => buf.extend(n.to_le_bytes().iter()),
                StructDataTypes::I64(n) => buf.extend(n.to_le_bytes().iter()),
                StructDataTypes::F64(n) => buf.extend(n.to_le_bytes().iter()),
            };
        }
        return buf
    }
}

pub fn data_type_as_f64(data: StructDataTypes) -> Option<f64> {
    if let StructDataTypes::F64(v) = data {
        return Some(v)
    }
    return None
}

pub fn data_type_as_i64(data: StructDataTypes) -> Option<i64> {
    if let StructDataTypes::I64(v) = data {
        return Some(v)
    }
    return None
}

pub fn data_type_as_bool(data: StructDataTypes) -> Option<bool> {
    if let StructDataTypes::Bool(v) = data {
        return Some(v)
    }
    return None
}