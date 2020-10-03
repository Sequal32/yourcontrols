pub mod memwriter;

use bimap::{self, BiHashMap};
use simconnect::SimConnector;
use std::{collections::{HashMap, VecDeque}};
use memwriter::MemWriter;
#[derive(Debug, Copy, Clone)]
pub struct LVar {
    pub integer: i32,
    pub floating: f64,
    // pub string: std::ffi::CString
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

pub struct LVars {
    requests: BiHashMap<String, u32>,
    next_request_id: u32,
    request_id: u32,
}

// SEND/RECEIVE define/client data ids
const LVARSEND: u32 = 0;
const LVARRECEIVE: u32 = 1;

impl LVars {
    pub fn new(request_id: u32) -> Self {
        Self {
            requests: BiHashMap::new(),
            request_id,
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
        let request_id = self.map_request(var_name);

        // Use custom memory writer to write request
        let mut writer = MemWriter::new(128, 4).unwrap();
        writer.write_u32(request_id);
        writer.pad(4);

        // Units are not required for execute_calculator_code
        if let Some(unit) = var_units {
            writer.write_string(format!(r#"({}, {})"#, var_name, unit.trim()));
        } else {
            writer.write_string(format!(r#"({})"#, var_name.trim()));
        }

        // Send the request
        conn.set_client_data(LVARSEND, LVARSEND, 0, 0, 128, writer.get_data_location() as *mut std::ffi::c_void);
        writer.deallocate();
    }

    pub fn set(&mut self, conn: &SimConnector, var_name: &str, var_units: Option<&str>, val: &str) {
        self.map_request(var_name);
        
        let mut writer = MemWriter::new(128, 4).unwrap();
        writer.write_i32(0);
        writer.pad(4);

        if let Some(unit) = var_units {
            writer.write_string(format!(r#"{} (>{}, {})"#, val.trim(), var_name.trim(), unit.trim()));
        } else {
            writer.write_string(format!(r#"{} (>{})"#, val.trim(), var_name.trim()));
        }

        conn.set_client_data(LVARSEND, LVARSEND, 0, 0, 128, writer.get_data_location() as *mut std::ffi::c_void);

        writer.deallocate();
    }

    pub fn process_client_data(&mut self, data: &simconnect::SIMCONNECT_RECV_CLIENT_DATA) -> Option<GetResult> {
        unsafe {
            if data._base.dwDefineID != LVARRECEIVE {return None}
            let data: ReceiveData = std::mem::transmute_copy(&data._base.dwData);
            return Some(
                GetResult {
                    var_name: self.get_request_string(data.request_id).clone(),
                    var: LVar {
                        integer: data.i,
                        floating: data.f,
                        // string: data.s
                    }
                }
            );
        }
    }

    pub fn on_connected(&self, conn: &SimConnector) {
        // Assign named data area to a client id
        conn.map_client_data_name_to_id("LVARSEND", LVARSEND);
        conn.map_client_data_name_to_id("LVARRECEIVE", LVARRECEIVE);
        // Define layout of client data area
        conn.add_to_client_data_definition(LVARSEND, 0, 4, 0.0, 0);
        conn.add_to_client_data_definition(LVARSEND, 4, 124, 0.0, 1);

        conn.add_to_client_data_definition(LVARRECEIVE, 0, 4, 0.0, 0);
        conn.add_to_client_data_definition(LVARRECEIVE, 4, 4, 0.0, 1);
        conn.add_to_client_data_definition(LVARRECEIVE, 8, 8, 0.0, 2);
        // conn.add_to_client_data_definition(LVARRECEIVE, 16, 112, 0.0, 3);
        // Create data area that other clients can read/write
        conn.create_client_data(LVARSEND, 128, simconnect::SIMCONNECT_CREATE_CLIENT_DATA_FLAG_DEFAULT);
        conn.create_client_data(LVARRECEIVE, 128, simconnect::SIMCONNECT_CREATE_CLIENT_DATA_FLAG_DEFAULT);
        // Listen for data written onto the RECEIVE data area
        conn.request_client_data(LVARRECEIVE, self.request_id, LVARRECEIVE, simconnect::SIMCONNECT_CLIENT_DATA_PERIOD_SIMCONNECT_CLIENT_DATA_PERIOD_ON_SET, 0, 0, 0, 0);
    }
}

pub struct DiffChecker<A, B> {
    indexes: HashMap<A, B>
}

impl<A, B> DiffChecker<A, B> where 
    A: std::cmp::Eq + std::hash::Hash + std::clone::Clone, 
    B: std::cmp::PartialEq
    {
    pub fn new() -> Self {
        Self {
            indexes: HashMap::new(),
        }
    }

    pub fn record(&mut self, index: &A, value: B) {
        self.indexes.insert(index.clone(), value);
    }

    pub fn record_and_is_diff(&mut self, index: &A, value: B) -> bool {
        let was_diff = self.is_diff(index, &value);
        self.record(index, value);
        return !was_diff.unwrap_or(false);
    }

    pub fn is_diff(&self, index: &A, value: &B) -> Option<bool> {
        let v =  self.indexes.get(index)?;
        return Some(*v == *value);
    } 

    pub fn get_all(&self) -> &HashMap<A,B> {
        return &self.indexes;
    }
}