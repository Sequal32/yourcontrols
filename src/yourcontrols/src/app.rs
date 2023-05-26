use crate::simconfig;

use base64::Engine;
use crossbeam_channel::{unbounded, Receiver, TryRecvError};
use laminar::Metrics;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::File;
use std::{io::Read, net::IpAddr};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering::SeqCst},
        Arc, Mutex,
    },
    thread,
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ConnectionMethod {
    Direct,
    Relay,
    CloudServer,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AppMessage {
    // Name, IsIPV6, port
    StartServer {
        username: String,
        is_ipv6: bool,
        use_upnp: bool,
        port: u16,
        method: ConnectionMethod,
    },
    Connect {
        username: String,
        session_id: Option<String>,
        isipv6: bool,
        ip: Option<IpAddr>,
        hostname: Option<String>,
        port: Option<u16>,
        method: ConnectionMethod,
    },
    TransferControl {
        target: String,
    },
    SetObserver {
        target: String,
        is_observer: bool,
    },
    LoadAircraft {
        config_file_name: String,
    },
    Disconnect,
    Startup,
    RunUpdater,
    ForceTakeControl,
    UpdateConfig {
        new_config: simconfig::Config,
    },
    GoObserver
}

fn get_message_str(type_string: &str, data: &str) -> String {
    format!(
        r#"MessageReceived({})"#,
        serde_json::json!({"type": type_string, "data": data})
    )
}

pub struct App {
    app_handle: Arc<Mutex<Option<web_view::Handle<i32>>>>,
    exited: Arc<AtomicBool>,
    rx: Receiver<AppMessage>,
}

impl App {
    pub fn setup(title: String) -> Self {
        let (tx, rx) = unbounded();

        let mut logo = vec![];
        File::open("assets/logo.png")
            .unwrap()
            .read_to_end(&mut logo)
            .ok();

        let handle = Arc::new(Mutex::new(None));
        let handle_clone = handle.clone();
        let exited = Arc::new(AtomicBool::new(false));
        let exited_clone = exited.clone();

        thread::spawn(move || {
            let webview = web_view::builder()
                .title(&title)
                .content(web_view::Content::Html(format!(
                    r##"<!DOCTYPE html>
                <html>
                <head>
                    <style>
                        {bootstrapcss}
                        {css}
                    </style>
                </head>
                    <body class="themed">
                    <img src="data:image/png;base64,{logo}" class="logo-image"/>
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
                    logo = base64::engine::general_purpose::STANDARD_NO_PAD.encode(logo.as_slice())
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

            webview.run().ok();
            exited_clone.store(true, SeqCst);
        });

        // Run
        Self {
            app_handle: handle,
            exited,
            rx,
        }
    }

    pub fn exited(&self) -> bool {
        self.exited.load(SeqCst)
    }

    pub fn get_next_message(&self) -> Result<AppMessage, TryRecvError> {
        self.rx.try_recv()
    }

    pub fn invoke(&self, type_string: &str, data: Option<&str>) {
        let handle = self.app_handle.lock().unwrap();
        if handle.is_none() {
            return;
        }
        // Send data to javascript
        let data = data.unwrap_or_default().to_string();
        let type_string = type_string.to_owned();
        handle
            .as_ref()
            .unwrap()
            .dispatch(move |webview| {
                webview
                    .eval(get_message_str(type_string.as_str(), data.as_str()).as_str())
                    .ok();
                Ok(())
            })
            .ok();
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

    pub fn server_started(&self) {
        self.invoke("server", None);
    }

    pub fn set_session_code(&self, code: &str) {
        self.invoke("session", Some(code));
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

    pub fn send_config(&self, value: &str) {
        self.invoke("config_msg", Some(value));
    }

    pub fn send_network(&self, metrics: &Metrics) {
        self.invoke(
            "metrics",
            Some(
                json!({
                    "sentPackets": metrics.sent_packets,
                    "receivePackets": metrics.received_packets,
                    "sentBandwidth": metrics.sent_kbps,
                    "receiveBandwidth": metrics.receive_kbps,
                    "packetLoss": metrics.packet_loss,
                    "ping": metrics.rtt/2.0
                })
                .to_string()
                .as_str(),
            ),
        )
    }

    pub fn set_host(&self) {
        self.invoke("host", None);
    }
}
