use crate::util::InDataTypes;
use byteorder::{LittleEndian, ReadBytesExt};
use std::{
    collections::HashMap,
    io::{self, Cursor, ErrorKind},
};
use yourcontrols_types::VarReaderTypes;

struct DefinitionEntry {
    data_type: InDataTypes,
    data_name: String,
}

pub struct VarReader {
    datum_id_map: HashMap<String, u32>,
    data_map: Vec<DefinitionEntry>,
}

pub type SimValue = HashMap<String, VarReaderTypes>;

// READ TAGGED DATA
impl VarReader {
    pub fn new() -> Self {
        Self {
            datum_id_map: HashMap::new(),
            data_map: Vec::new(),
        }
    }

    pub fn add_definition(&mut self, data_name: &str, data_type: InDataTypes) -> u32 {
        let datum_id = self.data_map.len() as u32;

        self.datum_id_map.insert(data_name.to_string(), datum_id);
        self.data_map.push(DefinitionEntry {
            data_type,
            data_name: data_name.to_string(),
        });

        datum_id
    }

    pub fn read_from_bytes(
        &self,
        item_count: u32,
        start: *const u32,
    ) -> Result<SimValue, io::Error> {
        let mut return_data: SimValue = HashMap::new();
        let mut current_pos = start;

        for _ in 0..item_count {
            unsafe {
                let mut buf: Vec<u8> = vec![];

                // Read all relevant bytes into buffer
                for i in 0..3 {
                    buf.extend_from_slice(&current_pos.offset(i).read().to_le_bytes())
                }

                let mut cursor = Cursor::new(buf);

                // Read data id
                let datum_id = cursor.read_u32::<LittleEndian>()?;
                // Get the matching data mapped to the datum id
                let data = match self.data_map.get(datum_id as usize) {
                    Some(d) => d,
                    None => {
                        return Err(io::Error::new(
                            ErrorKind::NotFound,
                            "DatumID wasn't defined.",
                        ))
                    }
                };

                let result_data;
                let data_size;

                match data.data_type {
                    InDataTypes::Bool => {
                        result_data = VarReaderTypes::Bool(std::mem::transmute_copy(
                            &cursor.read_i32::<LittleEndian>()?,
                        ));
                        data_size = 1;
                    }
                    InDataTypes::I32 => {
                        result_data = VarReaderTypes::I32(cursor.read_i32::<LittleEndian>()?);
                        data_size = 1;
                    }
                    InDataTypes::I64 => {
                        result_data = VarReaderTypes::I64(cursor.read_i64::<LittleEndian>()?);
                        data_size = 2;
                    }
                    InDataTypes::F64 => {
                        result_data = VarReaderTypes::F64(cursor.read_f64::<LittleEndian>()?);
                        data_size = 2;
                    }
                };

                return_data.insert(data.data_name.clone(), result_data);

                current_pos = current_pos.offset(data_size + 1);
            }
        }

        Ok(return_data)
    }

    pub fn write_to_data(&self, data: &HashMap<String, VarReaderTypes>) -> Vec<u8> {
        let mut buf: Vec<u8> = vec![];

        for (data_name, data) in data.iter() {
            let datum_id = self.datum_id_map[data_name];

            buf.extend(datum_id.to_le_bytes().iter());

            match data {
                VarReaderTypes::Bool(n) => buf.extend((*n as i64).to_le_bytes().iter()),
                VarReaderTypes::I32(n) => buf.extend((*n as i64).to_le_bytes().iter()),
                VarReaderTypes::I64(n) => buf.extend(n.to_le_bytes().iter()),
                VarReaderTypes::F64(n) => buf.extend(n.to_le_bytes().iter()),
            };
        }

        buf
    }

    #[cfg(test)]
    pub fn get_number_definitions(&self) -> u32 {
        self.data_map.len() as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::memwriter::MemWriter;

    #[test]
    fn test_read() {
        let mut definitions = VarReader::new();

        definitions.add_definition("PLANE LATITUDE", InDataTypes::F64);
        definitions.add_definition("PLANE LONGITUDE", InDataTypes::F64);

        let mut writer = MemWriter::new(64, 4).unwrap();
        writer.write_i32(0);
        writer.write_f64(42.0);
        writer.write_i32(1);
        writer.write_f64(128.0);

        let value = definitions
            .read_from_bytes(
                definitions.get_number_definitions(),
                writer.get_data_location() as *const u32,
            )
            .unwrap();
        assert_eq!(value["PLANE LATITUDE"], VarReaderTypes::F64(42.0));
        assert_eq!(value["PLANE LONGITUDE"], VarReaderTypes::F64(128.0));

        // Test a mix of datatypes
        definitions.add_definition("ELT ACTIVATED", InDataTypes::Bool);
        definitions.add_definition("Some enum", InDataTypes::I32);
        definitions.add_definition("Some big enum", InDataTypes::I64);

        writer.write_i32(2);
        writer.write_bool(false);
        writer.write_i32(3);
        writer.write_i32(1);
        writer.write_i32(4);
        writer.write_i64(3);

        let value = definitions
            .read_from_bytes(
                definitions.get_number_definitions(),
                writer.get_data_location() as *const u32,
            )
            .unwrap();

        assert_eq!(value["ELT ACTIVATED"], VarReaderTypes::Bool(false));
        assert_eq!(value["Some enum"], VarReaderTypes::I32(1));
        assert_eq!(value["Some big enum"], VarReaderTypes::I64(3));

        // Test bad data
        writer.write_i32(100);
        writer.write_bool(false);

        assert!(definitions
            .read_from_bytes(
                definitions.get_number_definitions() + 1,
                writer.get_data_location() as *const u32
            )
            .is_err());
    }

    #[test]
    fn test_write_and_read_back() {
        let mut definitions = VarReader::new();

        definitions.add_definition("PLANE LATITUDE", InDataTypes::F64);
        definitions.add_definition("PLANE LONGITUDE", InDataTypes::F64);

        let mut new_data = HashMap::new();
        new_data.insert("PLANE LATITUDE".to_string(), VarReaderTypes::F64(42.0));
        new_data.insert("PLANE LONGITUDE".to_string(), VarReaderTypes::F64(128.0));

        let data = definitions.write_to_data(&new_data);

        // Read
        let values = definitions
            .read_from_bytes(
                definitions.get_number_definitions(),
                data.as_ptr() as *const u32,
            )
            .unwrap();
        assert_eq!(values["PLANE LATITUDE"], VarReaderTypes::F64(42.0));
        assert_eq!(values["PLANE LONGITUDE"], VarReaderTypes::F64(128.0));
    }
}
