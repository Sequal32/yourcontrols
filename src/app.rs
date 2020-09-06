use base64;
use crossbeam_channel::{Receiver, unbounded};
use dns_lookup::lookup_host;
use std::{str::FromStr, net::Ipv4Addr, io::Read};
use std::fs::File;
use std::{sync::{Mutex, Arc, atomic::{AtomicBool, Ordering::SeqCst}}, thread};
use serde_json::Value;

pub enum AppMessage {
    Server(u16),
    Connect(Ipv4Addr, String, u16),
    Disconnect,
    TakeControl,
    RelieveControl,
    Startup,
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
    pub rx: Receiver<AppMessage>
}

fn get_ip_from_data(data: &Value) -> Result<Ipv4Addr, String> {
    match data.get("ip") {
        // Parse ip string as Ipv4Addr
        Some(ip_str) => match Ipv4Addr::from_str(ip_str.as_str().unwrap()) {
            Ok(ip) => Ok(ip),
            Err(_) => Err("Could not parse ip.".to_string())
        }
        None => match data.get("hostname") {
            // Resolve hostname
            Some(hostname_str) => match lookup_host(hostname_str.as_str().unwrap()) {
                Ok(hostnames) => match hostnames.iter().filter(|ip| {ip.is_ipv4()}).nth(0) {
                    // Only accept ipv4
                    Some(std::net::IpAddr::V4(ip)) => Ok(ip.clone()),
                    _ => Err("No Ipv4 addresses resolved to the specified hostname.".to_string())
                }
                Err(e) => Err(e.to_string())
            }
            None => Err("Invalid data passed.".to_string())
        }
    }
}

impl App {
    pub fn setup() -> Self {
        let (tx, rx) = unbounded();

        let mut logo = vec![];
        File::open("assets/logo.png").unwrap().read_to_end(&mut logo).ok();

        let handle = Arc::new(Mutex::new(None));
        let handle_clone = handle.clone();
        let exited = Arc::new(AtomicBool::new(false));
        let exited_clone = exited.clone();

        thread::spawn(move || {
            let webview = web_view::builder()
            .title("Shared Cockpit")
            .content(web_view::Content::Html(format!(r#"<!DOCTYPE html>
                <html>
                <head>
                    <link rel="stylesheet" href="https://stackpath.bootstrapcdn.com/bootstrap/4.5.2/css/bootstrap.min.css" integrity="sha384-JcKb8q3iqJ61gNV9KGb8thSsNjpSL0n8PARn9HuZOnIxN0hoP+VmmDGMN5t9UJ0Z" crossorigin="anonymous">
                    <style>{css}</style>
                </head>
                <body>{body}</body>
                <script>{js}</script>
                <img src="{logo}", class="logo-image"/>
                </html>
            "#, 
            css = include_str!("../web/stylesheet.css"), 
            js = include_str!("../web/main.js"), 
            body = include_str!("../web/index.html"),
            logo = format!("data:image/png;base64,{}", base64::encode(logo.as_slice()))
            )))

            .invoke_handler(move |web_view, arg| {
                let data: serde_json::Value = serde_json::from_str(arg).unwrap();
                match data["type"].as_str().unwrap() {
                    "connect" => {
                        match get_ip_from_data(&data) {
                            Ok(ip) => {
                                tx.send(
                                    AppMessage::Connect(
                                        ip, 
                                        if data.get("ip").is_some() {data["ip"].as_str().unwrap().to_string()} else {data["hostname"].as_str().unwrap().to_string()}, 
                                        data["port"].as_u64().unwrap() as u16)
                                    ).ok();
                                },
                            Err(e) => {
                                web_view.eval(
                                    get_message_str("client_fail", format!("Invalid IP or hostname. Reason: {}", e).as_str()).as_str()
                                ).ok();
                            }
                        };
                    },
                    "disconnect" => {tx.send(AppMessage::Disconnect).ok();},
                    "server" => {tx.send(AppMessage::Server(data["port"].as_u64().unwrap() as u16)).ok();},
                    "relieve" => {tx.send(AppMessage::RelieveControl).ok();},
                    "take" => {tx.send(AppMessage::TakeControl).ok();}
                    "startup" => {tx.send(AppMessage::Startup).ok();}
                    _ => ()
                };

                Ok(())
            })
            .user_data(0)
            .resizable(false)
            .size(600, 400)
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
            exited: exited,
            rx
        }
    }

    pub fn exited(&self) -> bool {
        return self.exited.load(SeqCst);
    }

    pub fn invoke(&mut self, type_string: &str, data: Option<&str>) {
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

    pub fn error(&mut self, msg: &str) {
        self.invoke("error", Some(msg));
    }

    pub fn set_port(&mut self, port: u16) {
        self.invoke("set_port", Some(port.to_string().as_str()));
    }

    pub fn set_ip(&mut self, ip: &str) {
        self.invoke("set_ip", Some(ip));
    }

    pub fn attempt(&mut self) {
        self.invoke("attempt", None);
    }

    pub fn connected(&mut self) {
        self.invoke("connected", None);
    }

    pub fn disconnected(&mut self) {
        self.invoke("disconnected", None);
    }

    pub fn server_fail(&mut self, reason: &str) {
        self.invoke("server_fail", Some(reason));
    }

    pub fn client_fail(&mut self, reason: &str) {
        self.invoke("client_fail", Some(reason));
    }

    pub fn gain_control(&mut self) {
        self.invoke("control", None);
    }

    pub fn lose_control(&mut self) {
        self.invoke("lostcontrol", None);
    }

    pub fn can_take_control(&mut self) {
        self.invoke("controlavail", None);
    }

    pub fn server_started(&mut self, client_count: u16) {
        self.invoke("server", Some(client_count.to_string().as_str()));
    }

    pub fn overloaded(&mut self) {
        self.invoke("overloaded", None);
    }

    pub fn stable(&mut self) {
        self.invoke("stable", None);
    }
}