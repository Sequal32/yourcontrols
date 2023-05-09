use super::memwriter::MemWriter;
use serde::Deserialize;
use simconnect::SimConnector;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LVar {
    pub integer: i32,
    pub floating: f64,
}

impl PartialEq for LVar {
    fn eq(&self, other: &Self) -> bool {
        self.floating == other.floating
    }
}

#[derive(Debug)]
pub struct GetResult {
    pub var_name: String,
    pub value: f64,
}

#[derive(Debug)]
struct DatumData {
    friendly_name: String,
    calculator: String,
}

#[repr(C)]
#[derive(Debug)]
struct ReturnDatum {
    id: i32,
    value: f64,
}

#[derive(Debug)]
struct InterpolateMapping {
    datum_id: u32,
    interpolation_type: InterpolationType,
    exec_string: String,
}

#[derive(Debug)]
pub struct InterpolateData {
    pub name: String,
    pub value: f64,
    pub time: f64,
}

#[derive(Deserialize, Debug)]
pub enum InterpolationType {
    Default,
    Wrap180,
    Wrap90,
    Wrap360,
    Invert,
    DefaultConstant,
    InvertConstant,
}

fn format_get(var_name: &str, var_units: Option<&str>) -> String {
    if let Some(unit) = var_units {
        format!(r#"({}, {})"#, var_name, unit.trim())
    } else {
        format!(r#"({})"#, var_name.trim())
    }
}

pub struct GaugeCommunicator {
    datums: Vec<DatumData>,
    interpolate_datums: HashMap<String, InterpolateMapping>,
}

// SEND/RECEIVE define/client data ids
const SEND: u32 = 0;
// const RECEIVE: u32 = 1;
const SEND_MULTIPLE: u32 = 2;
const RECEIVE_MULTIPLE: u32 = 3;
const MAP_INTERPOLATE: u32 = 4;
const SEND_INTERPOLATE: u32 = 5;

impl GaugeCommunicator {
    pub fn new() -> Self {
        Self {
            datums: Vec::new(),
            interpolate_datums: HashMap::new(),
        }
    }

    pub fn set(&self, conn: &SimConnector, var_name: &str, var_units: Option<&str>, val: &str) {
        let mut writer = MemWriter::new(128, 4).unwrap();
        writer.write_u32(0);
        writer.pad(4);

        if let Some(unit) = var_units {
            writer.write_string(format!(
                r#"{} (>{}, {})"#,
                val.trim(),
                var_name.trim(),
                unit.trim()
            ));
        } else {
            writer.write_string(format!(r#"{} (>{})"#, val.trim(), var_name.trim()));
        }

        unsafe {
            conn.set_client_data(
                SEND,
                SEND,
                0,
                0,
                128,
                writer.get_data_location() as *mut std::ffi::c_void,
            );
        }
    }

    pub fn send_raw(&self, conn: &SimConnector, string: &str) {
        let mut writer = MemWriter::new(128, 4).unwrap();
        writer.write_i32(0);
        writer.pad(4);
        writer.write_str(string);

        unsafe {
            conn.set_client_data(
                SEND,
                SEND,
                0,
                0,
                128,
                writer.get_data_location() as *mut std::ffi::c_void,
            );
        }
    }

    pub fn add_definition(&mut self, var_name: String, var_units: Option<&str>) {
        self.add_definition_raw(format_get(&var_name, var_units), var_name);
    }

    pub fn add_definition_raw(&mut self, calculator: String, name: String) {
        self.datums.push(DatumData {
            friendly_name: name,
            calculator,
        });
    }

    pub fn send_definitions(&mut self, conn: &SimConnector) {
        for x in 0..(self.datums.len() as f32 / 126_f32).ceil() as i32 {
            let mut writer = MemWriter::new(8096, 4).unwrap();

            for i in 0..126 {
                if let Some(datum_data) = self.datums.get((x * 126 + i) as usize) {
                    writer.write_str(&datum_data.calculator);
                    writer.pad(64 - datum_data.calculator.len() as isize);
                }
            }

            unsafe {
                conn.set_client_data(
                    SEND_MULTIPLE,
                    SEND_MULTIPLE,
                    0,
                    0,
                    8064,
                    writer.get_data_location() as *mut std::ffi::c_void,
                );
            }
        }
    }

    pub fn get_number_defined(&self) -> usize {
        self.datums.len()
    }

    // Aircraft vars only
    pub fn add_interpolate_mapping(
        &mut self,
        calculator_var_name: &str,
        index_var_name: String,
        var_units: Option<&str>,
        interpolation_type: InterpolationType,
    ) {
        let exec_string = match var_units {
            Some(unit) => format!(r#"(>{}, {})"#, calculator_var_name.trim(), unit.trim()),
            None => format!(r#"(>{})"#, calculator_var_name.trim()),
        };

        self.interpolate_datums.insert(
            index_var_name,
            InterpolateMapping {
                datum_id: self.interpolate_datums.len() as u32,
                interpolation_type,
                exec_string,
            },
        );
    }

    pub fn send_new_interpolation_data(
        &self,
        conn: &SimConnector,
        time: f64,
        data: &[InterpolateData],
    ) {
        let mut writer = MemWriter::new(2048, 8).unwrap();
        let mut count = 0;

        writer.write_u32(100);
        writer.write_f64(time);

        for entry in data {
            if let Some(datum) = self.interpolate_datums.get(&entry.name) {
                writer.write_u32(datum.datum_id);
                writer.write_f64(entry.value);

                count += 1;
            }
        }

        unsafe {
            conn.set_client_data(
                SEND_INTERPOLATE,
                SEND_INTERPOLATE,
                simconnect::SIMCONNECT_CLIENT_DATA_SET_FLAG_TAGGED,
                0,
                count * 12 + 12,
                writer.get_data_location() as *mut std::ffi::c_void,
            );
        }
    }

    fn do_operation(&self, operation: i32, conn: &SimConnector) {
        let mut writer = MemWriter::new(128, 4).unwrap();
        writer.write_i32(operation);

        unsafe {
            conn.set_client_data(
                SEND,
                SEND,
                0,
                0,
                128,
                writer.get_data_location() as *mut std::ffi::c_void,
            );
        }
    }

    fn clear_definitions(&mut self, conn: &SimConnector) {
        self.do_operation(-1, conn);
    }

    pub fn stop_interpolation(&self, conn: &SimConnector) {
        self.do_operation(-2, conn);
    }

    pub fn process_client_data(
        &mut self,
        data: &simconnect::SIMCONNECT_RECV_CLIENT_DATA,
    ) -> Vec<GetResult> {
        let datums: &'static [ReturnDatum] = unsafe {
            let pointer = std::ptr::addr_of!(data._base.dwData);
            let array_pointer = pointer.add(2) as *const ReturnDatum;
            let length = pointer.read();

            std::slice::from_raw_parts(array_pointer, length as usize)
        };

        let mut result = Vec::new();

        for datum in datums {
            if let Some(datum_data) = self.datums.get(datum.id as usize) {
                let value = datum.value.clamp(-10e64f64, 10e64f64);
                result.push(GetResult {
                    var_name: datum_data.friendly_name.clone(),
                    value,
                })
            }
        }

        result
    }

    fn write_interpolate_mapping(&mut self, conn: &SimConnector) {
        let mut writer = MemWriter::new(8096, 4).unwrap();

        for (_, datum) in self.interpolate_datums.iter() {
            let interpolation_type_int = match datum.interpolation_type {
                InterpolationType::Default => 0,
                InterpolationType::Wrap180 => 1,
                InterpolationType::Wrap360 => 2,
                InterpolationType::Wrap90 => 3,
                InterpolationType::Invert => 4,
                InterpolationType::DefaultConstant => 5,
                InterpolationType::InvertConstant => 6,
            };

            writer.write_u32(datum.datum_id);
            writer.write_u32(interpolation_type_int);
            writer.write_str(&datum.exec_string);
            writer.pad(64 - datum.exec_string.len() as isize);
        }

        unsafe {
            conn.set_client_data(
                MAP_INTERPOLATE,
                MAP_INTERPOLATE,
                simconnect::SIMCONNECT_CLIENT_DATA_SET_FLAG_TAGGED,
                0,
                (self.interpolate_datums.len() * 72) as u32,
                writer.get_data_location() as *mut std::ffi::c_void,
            );
        }
    }

    pub fn on_connected(&mut self, conn: &SimConnector) {
        // Assign named data area to a client id
        conn.map_client_data_name_to_id("YCSEND", SEND);
        // conn.map_client_data_name_to_id("YCRECEIVE", RECEIVE);
        conn.map_client_data_name_to_id("YCSENDMULTI", SEND_MULTIPLE);
        conn.map_client_data_name_to_id("YCRECEIVEMULTI", RECEIVE_MULTIPLE);
        conn.map_client_data_name_to_id("YCMAPINTERPOLATE", MAP_INTERPOLATE);
        conn.map_client_data_name_to_id("YCSENDINTERPOLATE", SEND_INTERPOLATE);

        // Define layout of client data area
        conn.add_to_client_data_definition(SEND, 0, 4, 0.0, 0);
        conn.add_to_client_data_definition(SEND, 4, 124, 0.0, 1);

        conn.add_to_client_data_definition(SEND_INTERPOLATE, 0, 8, 0.0, 100);

        for i in 0..100 {
            conn.add_to_client_data_definition(MAP_INTERPOLATE, i * 68, 68, 0.0, i);
            conn.add_to_client_data_definition(SEND_INTERPOLATE, i * 8 + 8, 8, 0.0, i);
        }

        conn.add_to_client_data_definition(RECEIVE_MULTIPLE, 0, 8096, 0.0, 0);
        conn.add_to_client_data_definition(SEND_MULTIPLE, 0, 8064, 0.0, 0);
        // Clear gauge definitions
        self.clear_definitions(conn);

        conn.request_client_data(
            RECEIVE_MULTIPLE,
            1,
            RECEIVE_MULTIPLE,
            simconnect::SIMCONNECT_CLIENT_DATA_PERIOD_SIMCONNECT_CLIENT_DATA_PERIOD_ON_SET,
            0,
            0,
            0,
            0,
        );
        // Write some data
        self.write_interpolate_mapping(conn);
    }
}
