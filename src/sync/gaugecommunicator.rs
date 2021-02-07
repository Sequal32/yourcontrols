use bimap::{self, BiHashMap};
use serde::Deserialize;
use simconnect::SimConnector;
use std::{collections::HashMap, io::{Cursor}};
use super::memwriter::MemWriter;
use {byteorder::{ReadBytesExt, LittleEndian}};

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
#[repr(C)]
pub struct GetResult {
    pub var_name: String,
    pub var: LVar
}

#[repr(C)]
#[derive(Debug)]
struct ReceiveData {
    request_id: u32,
    i: i32,
    f: f64,
    // s: std::ffi::CString
}

#[derive(Debug)]
struct InterpolateMapping {
    datum_id: u32,
    interpolation_type: InterpolationType,
    exec_string: String
}

#[derive(Debug)]
pub struct InterpolateData {
    pub name: String,
    pub value: f64,
    pub time: f64
}

#[derive(Deserialize, Debug)]
pub enum InterpolationType {
    Default,
    Wrap180,
    Wrap90,
    Wrap360,
    Invert
}

fn format_get(var_name: &str, var_units: Option<&str>) -> String {
    if let Some(unit) = var_units {
        return format!(r#"({}, {})"#, var_name, unit.trim());
    } else {
        return format!(r#"({})"#, var_name.trim());
    }
}

pub struct GaugeCommunicator {
    requests: BiHashMap<String, u32>,
    datums: Vec<String>,
    interpolate_datums: HashMap<String, InterpolateMapping>,
    next_request_id: u32,
}

#[derive(Debug)]
pub enum LVarResult {
    Single(GetResult),
    Multi(Vec<GetResult>)
}

// SEND/RECEIVE define/client data ids
const SEND: u32 = 0;
const RECEIVE: u32 = 1;
const SEND_MULTIPLE: u32 = 2;
const RECEIVE_MULTIPLE: u32 = 3;
const MAP_INTERPOLATE: u32 = 4;
const SEND_INTERPOLATE: u32 = 5;

impl GaugeCommunicator {
    pub fn new() -> Self {
        Self {
            requests: BiHashMap::new(),
            datums: Vec::new(),
            interpolate_datums: HashMap::new(),
            next_request_id: 0
        }
    }

    fn map_request(&mut self, var_name: &str) -> u32 {
        if let Some(id) = self.requests.get_by_left(&var_name.to_string()) {
            return *id;
        } else {
            let id = self.next_request_id;

            self.requests.insert(var_name.to_string(), id);

            self.next_request_id += 1;
            return id;
        }
    }

    fn get_request_string(&self, request_id: u32) -> &String {
        return self.requests.get_by_right(&request_id).unwrap()
    }
    
    pub fn set(&mut self, conn: &SimConnector, var_name: &str, var_units: Option<&str>, val: &str) {	
        let mut writer = MemWriter::new(128, 4).unwrap();	
        writer.write_u32(self.map_request(var_name));	
        writer.pad(4);	
    
        if let Some(unit) = var_units {	
            writer.write_string(format!(r#"{} (>{}, {})"#, val.trim(), var_name.trim(), unit.trim()));	
        } else {	
            writer.write_string(format!(r#"{} (>{})"#, val.trim(), var_name.trim()));	
        }	
    
        conn.set_client_data(SEND, SEND, 0, 0, 128, writer.get_data_location() as *mut std::ffi::c_void);	
    
        writer.deallocate();	
    }	
    
    pub fn send_raw(&self, conn: &SimConnector, string: &str) {	
        let mut writer = MemWriter::new(128, 4).unwrap();	
        writer.write_i32(0);	
        writer.pad(4);	
        writer.write_str(string);	
        conn.set_client_data(SEND, SEND, 0, 0, 128, writer.get_data_location() as *mut std::ffi::c_void);	
        writer.deallocate();	
    }

    pub fn add_definition(&mut self, conn: &SimConnector, var_name: &str, var_units: Option<&str>) {
        self.add_definition_raw(conn, &format_get(var_name, var_units), var_name);
    }

    pub fn add_definition_raw(&mut self, conn: &SimConnector, string: &str, name: &str) {
        let mut writer = MemWriter::new(128, 4).unwrap();
        writer.write_str(string);

        self.datums.push(name.to_string());
        conn.add_to_client_data_definition(RECEIVE_MULTIPLE, (self.datums.len() * 16) as u32, 16, 0.0, self.datums.len() as u32);

        conn.set_client_data(SEND_MULTIPLE, SEND_MULTIPLE, 0, 0, 128, writer.get_data_location() as *mut std::ffi::c_void);
    }

    // Aircraft vars only
    pub fn add_interpolate_mapping(&mut self, calculator_var_name: &str, index_var_name: String, var_units: Option<&str>, interpolation_type: InterpolationType) {
        let exec_string = match var_units {
            Some(unit) => format!(r#"(>{}, {})"#, calculator_var_name.trim(), unit.trim()),
            None => format!(r#"(>{})"#, calculator_var_name.trim())
        };

        self.interpolate_datums.insert(index_var_name, InterpolateMapping {
            datum_id: self.interpolate_datums.len() as u32,
            interpolation_type,
            exec_string,
        });
    }

    pub fn send_new_interpolation_data(&self, conn: &SimConnector, time: f64, data: &Vec<InterpolateData>) {
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

        conn.set_client_data(SEND_INTERPOLATE, SEND_INTERPOLATE, simconnect::SIMCONNECT_CLIENT_DATA_SET_FLAG_TAGGED, 0, count * 12 + 12, writer.get_data_location() as *mut std::ffi::c_void);
    }

    fn read_multiple_data(&mut self, count: u32, pointer: *const u32) -> Result<Vec<GetResult>, std::io::Error> {
        let mut return_vars = Vec::new();

        unsafe {
            for i in 0..count {
    
                let mut buf = Vec::new();
                for x in 0..5 {
                    buf.extend_from_slice(&pointer.offset((i * 5 + x) as isize).read().to_le_bytes());
                }
                    //
        
                let mut cursor = Cursor::new(buf);
        
                let datum_id = cursor.read_i32::<LittleEndian>()?;
                let datum_name = match self.datums.get(datum_id as usize) {
                    Some(s) => s,
                    None => continue
                };
                // Skip 4 bytes for padding
                cursor.read_u32::<LittleEndian>().ok();
        
                return_vars.push(
                    GetResult {
                        var_name: datum_name.clone(),
                        var: LVar {
                            integer: cursor.read_i32::<LittleEndian>()?,
                            floating: cursor.read_f64::<LittleEndian>()?,
                        }
                    }
                );
            }
        }
        
        return Ok(return_vars);
    }

    fn do_operation(&self, operation: i32, conn: &SimConnector) {
        let mut writer = MemWriter::new(128, 4).unwrap();
        writer.write_i32(operation);
        conn.set_client_data(SEND, SEND, 0, 0, 128, writer.get_data_location() as *mut std::ffi::c_void);
    }

    fn clear_definitions(&mut self, conn: &SimConnector) {
        self.do_operation(-1, conn);
    }

    pub fn stop_interpolation(&self, conn: &SimConnector) {
        self.do_operation(-2, conn);
    }

    pub fn fetch_all(&self, conn: &SimConnector) {
        conn.request_client_data(RECEIVE_MULTIPLE, 1, RECEIVE_MULTIPLE, simconnect::SIMCONNECT_CLIENT_DATA_PERIOD_SIMCONNECT_CLIENT_DATA_PERIOD_ONCE, simconnect::SIMCONNECT_CLIENT_DATA_REQUEST_FLAG_TAGGED, 0, 0, 0);
    }

    pub fn process_client_data(&mut self, conn: &simconnect::SimConnector, data: &simconnect::SIMCONNECT_RECV_CLIENT_DATA) -> Option<LVarResult> {
        match data._base.dwDefineID {
            RECEIVE => unsafe {
                let data: ReceiveData = std::mem::transmute_copy(&data._base.dwData);
                return Some(
                    LVarResult::Single(
                        GetResult {
                            var_name: self.get_request_string(data.request_id).clone(),
                            var: LVar {
                                integer: data.i,
                                floating: data.f,
                                // string: data.s
                            }
                        }
                    )
                );
            }
            RECEIVE_MULTIPLE => {
                // Was initial fetch done
                if data._base.dwRequestID == 1 {
                    conn.request_client_data(RECEIVE_MULTIPLE, 0, RECEIVE_MULTIPLE, simconnect::SIMCONNECT_CLIENT_DATA_PERIOD_SIMCONNECT_CLIENT_DATA_PERIOD_ON_SET, simconnect::SIMCONNECT_CLIENT_DATA_REQUEST_FLAG_CHANGED | simconnect::SIMCONNECT_CLIENT_DATA_REQUEST_FLAG_TAGGED, 0, 0, 0);
                }

                unsafe {
                    let pointer = &data._base.dwData as *const u32;
                    match self.read_multiple_data(data._base.dwDefineCount as u32, pointer) {
                        Ok(d) => Some(LVarResult::Multi(d)),
                        Err(_) => None
                    }

                    
                }
            }
            _ => None
        }
    }

    fn write_interpolate_mapping(&mut self, conn: &SimConnector) {
        let mut writer = MemWriter::new(2048, 4).unwrap();

        for (_, datum) in self.interpolate_datums.iter() {

            let interpolation_type_int = match datum.interpolation_type {
                InterpolationType::Default => 0,
                InterpolationType::Wrap180 => 1,
                InterpolationType::Wrap90 => 2,
                InterpolationType::Wrap360 => 3,
                InterpolationType::Invert => 4
            };

            writer.write_u32(datum.datum_id);
            writer.write_u32(interpolation_type_int);
            writer.write_str(&datum.exec_string);
            writer.pad(64-datum.exec_string.len() as isize);
        }

        conn.set_client_data(MAP_INTERPOLATE, MAP_INTERPOLATE, simconnect::SIMCONNECT_CLIENT_DATA_SET_FLAG_TAGGED, 0, (self.interpolate_datums.len() * 72) as u32, writer.get_data_location() as *mut std::ffi::c_void);
    }

    pub fn on_connected(&mut self, conn: &SimConnector) {
        // Assign named data area to a client id
        conn.map_client_data_name_to_id("YCSEND", SEND);
        conn.map_client_data_name_to_id("YCRECEIVE", RECEIVE);
        conn.map_client_data_name_to_id("YCSENDMULTI", SEND_MULTIPLE);
        conn.map_client_data_name_to_id("YCRECEIVEMULTI", RECEIVE_MULTIPLE);
        conn.map_client_data_name_to_id("YCMAPINTERPOLATE", MAP_INTERPOLATE);
        conn.map_client_data_name_to_id("YCSENDINTERPOLATE", SEND_INTERPOLATE);
   
        // Define layout of client data area
        conn.add_to_client_data_definition(SEND, 0, 4, 0.0, 0);
        conn.add_to_client_data_definition(SEND, 4, 124, 0.0, 1);

        conn.add_to_client_data_definition(RECEIVE, 0, 4, 0.0, 0);
        conn.add_to_client_data_definition(RECEIVE, 4, 4, 0.0, 1);
        conn.add_to_client_data_definition(RECEIVE, 8, 8, 0.0, 2);

        conn.add_to_client_data_definition(SEND_MULTIPLE, 0, 128, 0.0, 0);
        conn.add_to_client_data_definition(RECEIVE_MULTIPLE, 0, 16, 0.0, 0);

        conn.add_to_client_data_definition(SEND_INTERPOLATE, 0, 8, 0.0, 100);

        for i in 0..30 {
            conn.add_to_client_data_definition(MAP_INTERPOLATE, i * 68, 68, 0.0, i);
            conn.add_to_client_data_definition(SEND_INTERPOLATE, i * 8 + 8, 8, 0.0, i);
        }
        // Listen for data written onto the RECEIVE data area
        conn.request_client_data(RECEIVE, 0, RECEIVE, simconnect::SIMCONNECT_CLIENT_DATA_PERIOD_SIMCONNECT_CLIENT_DATA_PERIOD_ON_SET, 0, 0, 0, 0);
        conn.request_client_data(RECEIVE_MULTIPLE, 0, RECEIVE_MULTIPLE, simconnect::SIMCONNECT_CLIENT_DATA_PERIOD_SIMCONNECT_CLIENT_DATA_PERIOD_ON_SET, simconnect::SIMCONNECT_CLIENT_DATA_REQUEST_FLAG_CHANGED | simconnect::SIMCONNECT_CLIENT_DATA_REQUEST_FLAG_TAGGED, 0, 0, 0);
        // Clear gauge definitions
        self.clear_definitions(conn);
        // Write some data
        self.write_interpolate_mapping(conn);
    }
}