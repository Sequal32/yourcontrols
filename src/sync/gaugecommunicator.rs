use super::memwriter::MemWriter;
use simconnect::SimConnector;

pub struct GaugeCommunicator {
}

// SEND/RECEIVE define/client data ids
const SEND: u32 = 0;

impl GaugeCommunicator {
    pub fn new() -> Self {
        Self {
        }
    }
    
    pub fn set(&mut self, conn: &SimConnector, var_name: &str, var_units: Option<&str>, val: &str) {	
        let mut writer = MemWriter::new(128, 4).unwrap();	
        writer.write_u32(0);	
        writer.pad(4);	
    
        if let Some(unit) = var_units {	
            writer.write_string(format!(r#"{} (>{}, {})"#, val.trim(), var_name.trim(), unit.trim()));	
        } else {
            writer.write_string(format!(r#"{} (>{})"#, val.trim(), var_name.trim()));	
        }	
    
        conn.set_client_data(SEND, SEND, 0, 0, 128, writer.get_data_location() as *mut std::ffi::c_void);	
    
        writer.deallocate();	
    }	

    pub fn on_connected(&mut self, conn: &SimConnector) {
        // Assign named data area to a client id
        conn.map_client_data_name_to_id("LVARSEND", SEND);
   
        // Define layout of client data area
        conn.add_to_client_data_definition(SEND, 0, 4, 0.0, 0);
        conn.add_to_client_data_definition(SEND, 4, 124, 0.0, 1);

        // Create data area that other clients can read/write
        conn.create_client_data(SEND, 128, simconnect::SIMCONNECT_CREATE_CLIENT_DATA_FLAG_DEFAULT);
    }
}