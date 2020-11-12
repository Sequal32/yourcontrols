use super::memwriter::MemWriter;
use {byteorder::{ReadBytesExt, LittleEndian}};
use bimap::{self, BiHashMap};
use simconnect::SimConnector;
use std::{io::{Cursor}};

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

fn format_get(var_name: &str, var_units: Option<&str>) -> String {
    if let Some(unit) = var_units {
        return format!(r#"({}, {})"#, var_name, unit.trim());
    } else {
        return format!(r#"({})"#, var_name.trim());
    }
}

pub struct LVars {
    requests: BiHashMap<String, u32>,
    datums: Vec<String>,
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

impl LVars {
    pub fn new() -> Self {
        Self {
            requests: BiHashMap::new(),
            datums: Vec::new(),
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

    pub fn get(&mut self, conn: &SimConnector, var_name: &str, var_units: Option<&str>) {	
        self.send_raw(conn, &format_get(var_name, var_units));	
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

    fn clear_definitions(&mut self, conn: &SimConnector) {
        let mut writer = MemWriter::new(128, 4).unwrap();
        writer.write_i32(-1);
        conn.set_client_data(SEND, SEND, 0, 0, 128, writer.get_data_location() as *mut std::ffi::c_void);
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

    pub fn on_connected(&mut self, conn: &SimConnector) {
        // Assign named data area to a client id
        conn.map_client_data_name_to_id("LVARSEND", SEND);
        conn.map_client_data_name_to_id("LVARRECEIVE", RECEIVE);
        conn.map_client_data_name_to_id("LVARSENDMULTI", SEND_MULTIPLE);
        conn.map_client_data_name_to_id("LVARRECEIVEMULTI", RECEIVE_MULTIPLE);
   
        // Define layout of client data area
        conn.add_to_client_data_definition(SEND, 0, 4, 0.0, 0);
        conn.add_to_client_data_definition(SEND, 4, 124, 0.0, 1);

        conn.add_to_client_data_definition(RECEIVE, 0, 4, 0.0, 0);
        conn.add_to_client_data_definition(RECEIVE, 4, 4, 0.0, 1);
        conn.add_to_client_data_definition(RECEIVE, 8, 8, 0.0, 2);

        conn.add_to_client_data_definition(SEND_MULTIPLE, 0, 128, 0.0, 0);
        conn.add_to_client_data_definition(RECEIVE_MULTIPLE, 0, 16, 0.0, 0);
        // conn.add_to_client_data_definition(LVARRECEIVE, 16, 112, 0.0, 3);
        // Create data area that other clients can read/write
        conn.create_client_data(SEND, 128, simconnect::SIMCONNECT_CREATE_CLIENT_DATA_FLAG_DEFAULT);
        conn.create_client_data(RECEIVE, 128, simconnect::SIMCONNECT_CREATE_CLIENT_DATA_FLAG_DEFAULT);
        conn.create_client_data(SEND_MULTIPLE, 128, 0);
        conn.create_client_data(RECEIVE_MULTIPLE, 8096, 0);
        // Listen for data written onto the RECEIVE data area
        conn.request_client_data(RECEIVE, 0, RECEIVE, simconnect::SIMCONNECT_CLIENT_DATA_PERIOD_SIMCONNECT_CLIENT_DATA_PERIOD_ON_SET, 0, 0, 0, 0);
        conn.request_client_data(RECEIVE_MULTIPLE, 0, RECEIVE_MULTIPLE, simconnect::SIMCONNECT_CLIENT_DATA_PERIOD_SIMCONNECT_CLIENT_DATA_PERIOD_ON_SET, simconnect::SIMCONNECT_CLIENT_DATA_REQUEST_FLAG_CHANGED | simconnect::SIMCONNECT_CLIENT_DATA_REQUEST_FLAG_TAGGED, 0, 0, 0);
        // Clear gauge definitions
        self.clear_definitions(conn);
    }
}