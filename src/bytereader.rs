use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor};
use std::ptr;
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

    pub fn read_from_bytes(&self, start: *const u32) -> IndexMap<String, StructDataTypes> {
        let mut return_data: IndexMap<String, StructDataTypes> = IndexMap::new();
        let mut current_pos = start;
        
        for (key, data_type) in &self.data_map {
            unsafe {
                let mut buf: Vec<u8> = vec![];
                buf.extend_from_slice(&ptr::read(current_pos).to_le_bytes());

                match data_type {
                    InDataTypes::I64 | InDataTypes::F64 => {buf.extend_from_slice(&ptr::read(current_pos.offset(1)).to_le_bytes());}
                    _ => ()
                }

                let mut cursor = Cursor::new(buf);

                match data_type {
                    InDataTypes::Bool => {
                        return_data.insert(key.to_string(), StructDataTypes::Bool(std::mem::transmute_copy(&cursor.read_i32::<LittleEndian>().unwrap())))
                    }
                    InDataTypes::I32 => {
                        return_data.insert(key.to_string(), StructDataTypes::I32(cursor.read_i32::<LittleEndian>().unwrap()))
                    }
                    InDataTypes::I64 => {
                        return_data.insert(key.to_string(), StructDataTypes::I64(cursor.read_i64::<LittleEndian>().unwrap()))
                    }
                    InDataTypes::F64 => {
                        return_data.insert(key.to_string(), StructDataTypes::F64(cursor.read_f64::<LittleEndian>().unwrap()))
                    }
                };

                current_pos = current_pos.offset(2);
            }
        }
        return return_data;
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