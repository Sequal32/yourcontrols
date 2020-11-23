use base64;
use crossbeam_channel::{Receiver, TryRecvError, unbounded};
use log::{info};
use std::{net::IpAddr, io::Read};
use std::fs::File;
use std::{sync::{Mutex, Arc, atomic::{AtomicBool, Ordering::SeqCst}}, thread};
use serde::{Serialize, Deserialize};
use crate::simconfig;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ConnectionMethod {
    Direct,
    UPnP,
    CloudServer
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AppMessage {
    // Name, IsIPV6, port
    StartServer {username: String, isipv6: bool, port: u16, method: ConnectionMethod},
    Connect {username: String, session_id: String, isipv6: bool, ip: Option<IpAddr>, hostname: Option<String>, port: Option<u16>, method: ConnectionMethod},
    TransferControl {target: String},
    SetObserver {target: String, is_observer: bool},
    LoadAircraft {config_file_name: String},
    Disconnect,
    Startup,
    RunUpdater,
    ForceTakeControl,
    UpdateConfig {new_config: simconfig::Config}
}

fn get_message_str(type_string: &str, data: &str) -> String {
    format!(
        r#"MessageReceived({})"#, 
        serde_json::json!({"type": type_string, "data": data}).to_string()
    )
}

pub struct App {
    app_handle: Arc<Mutex<Option<web_view::Handle<i32>>>>,
    exited: Arc<AtomicBool>,
    rx: Receiver<AppMessage>,
}

impl App {
    pub fn setup(title: String) -> Self {
        info!("Creating webview...");
        
        let (tx, rx) = unbounded();

        let mut logo = vec![];
        File::open("assets/logo.png").unwrap().read_to_end(&mut logo).ok();

        let handle = Arc::new(Mutex::new(None));
        let handle_clone = handle.clone();
        let exited = Arc::new(AtomicBool::new(false));
        let exited_clone = exited.clone();

        info!("Spawning webview thread...");

        thread::spawn(move || {
            let webview = web_view::builder()
            .title(&title)
            .content(web_view::Content::Html(format!(r##"<!DOCTYPE html>
                <html>
                <head>
                    <style>
                        {bootstrapcss}
                        {css}
                    </style>
                </head>
                    <body class="themed">
                    <img src="{logo}" class="logo-image"/>
                    {body}
                </body>
                <script>
                    {jquery}
                    {bootstrapjs}
                    {js1}
                    {js}
                </script>
                </html>
            "##,
            css = include_str!("../web/stylesheet.css"), 
            js = include_str!("../web/main.js"), 
            js1 = include_str!("../web/list.js"),
            body = include_str!("../web/index.html"),
            jquery = include_str!("../web/jquery.min.js"),
            bootstrapjs = include_str!("../web/bootstrap.bundle.min.js"),
            bootstrapcss = include_str!("../web/bootstrap.min.css"),
            logo = format!("data:image/png;base64,{}", base64::encode(logo.as_slice()))
            )))

            .invoke_handler(move |_, arg| {
                tx.try_send(serde_json::from_str(arg).unwrap()).ok();

                Ok(())
            })
            .user_data(0)
            .resizable(true)
            .size(1000, 800)
            .build()
            .unwrap();
            
            let mut handle = handle_clone.lock().unwrap();
            *handle = Some(webview.handle());
            std::mem::drop(handle);

            info!("Running webview thread...");

            webview.run().ok();
            exited_clone.store(true, SeqCst);
        });

        // Run
        Self {
            app_handle: handle,
            exited: exited,
            rx,
        }
    }

    pub fn exited(&self) -> bool {
        return self.exited.load(SeqCst);
    }

    pub fn get_next_message(&self) -> Result<AppMessage, TryRecvError> {
        return self.rx.try_recv();
    }

    pub fn invoke(&self, type_string: &str, data: Option<&str>) {
        let handle = self.app_handle.lock().unwrap();
        if handle.is_none() {return}
        // Send data to javascript
        let data = data.unwrap_or_default().to_string();
        let type_string = type_string.to_owned();
        handle.as_ref().unwrap().dispatch(move |webview| {
            webview.eval(get_message_str(type_string.as_str(), data.as_str()).as_str()).ok();
            Ok(())
        }).ok();
    }

    pub fn error(&self, msg: &str) {
        self.invoke("error", Some(msg));
    }

    pub fn attempt(&self) {
        self.invoke("attempt", None);
    }

    pub fn connected(&self) {
        self.invoke("connected", None);
    }

    pub fn server_fail(&self, reason: &str) {
        self.invoke("server_fail", Some(reason));
    }

    pub fn client_fail(&self, reason: &str) {
        self.invoke("client_fail", Some(reason));
    }

    pub fn gain_control(&self) {
        self.invoke("control", None);
    }

    pub fn lose_control(&self) {
        self.invoke("lostcontrol", None);
    }

    pub fn server_started(&self, client_count: u16, session_id: Option<&str>) {
        self.invoke("server", Some(&format!("{} clients connected. Session ID: {}", client_count, session_id.unwrap_or(""))));
    }

    pub fn new_connection(&self, name: &str) {
        self.invoke("newconnection", Some(name));
    }

    pub fn lost_connection(&self, name: &str) {
        self.invoke("lostconnection", Some(name));
    }

    pub fn observing(&self, observing: bool) {
        if observing {
            self.invoke("observing", None);
        } else {
            self.invoke("stop_observing", None);
        }
    }

    pub fn set_observing(&self, name: &str, observing: bool) {
        if observing {
            self.invoke("set_observing", Some(name));
        } else {
            self.invoke("set_not_observing", Some(name));
        }
    }

    pub fn set_incontrol(&self, name: &str) {
        self.invoke("set_incontrol", Some(name));
    }

    pub fn add_aircraft(&self, name: &str) {
        self.invoke("add_aircraft", Some(name));
    }

    pub fn version(&self, version: &str) {
        self.invoke("version", Some(version))
    }

    pub fn update_failed(&self) {
        self.invoke("update_failed", None);
    }

    pub fn send_config(&self, value: &str){
        self.invoke("config_msg", Some(value));
    }
}