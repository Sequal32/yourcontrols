use base64;
use std::fs::File;
use web_view::{self, WebView, };
use std::{str::FromStr, net::Ipv4Addr, io::Read};
use crossbeam_channel::{Sender, Receiver, unbounded};
use std::sync::{Mutex, Arc, atomic::{AtomicBool, Ordering::SeqCst}};

pub enum AppMessage {
    Server(u16),
    Connect(Ipv4Addr, u16),
    Disconnect,
    TakeControl,
    RelieveControl,
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

impl App {

    pub fn setup() -> Self {
        let (tx, rx) = unbounded();

        let mut logo = vec![];
        File::open("assets/logo.png").unwrap().read_to_end(&mut logo).ok();

        let handle = Arc::new(Mutex::new(None));
        let handle_clone = handle.clone();
        let exited = Arc::new(AtomicBool::new(false));
        let exited_clone = exited.clone();

        std::thread::spawn(move || {
            let webview = web_view::builder()
            .title("Shared Cockpit")
            .content(web_view::Content::Html(format!(r#"
            <!DOCTYPE html>
                <html>
                <head>
                    <link rel="stylesheet" href="https://stackpath.bootstrapcdn.com/bootstrap/4.5.2/css/bootstrap.min.css" integrity="sha384-JcKb8q3iqJ61gNV9KGb8thSsNjpSL0n8PARn9HuZOnIxN0hoP+VmmDGMN5t9UJ0Z" crossorigin="anonymous">
                    <style>{css}</style>
                </head>
                <body>{body}</body>
                <script>{js}</script>
                <image src={logo}, class="logo-image"/>
                </html>
            "#, 
            css = include_str!("../web/stylesheet.css"), 
            js = include_str!("../web/main.js"), 
            body = include_str!("../web/index.html"),
            logo = format!("data:image/png;base64,{}", base64::encode(logo.as_slice()))
            )))

            .invoke_handler(move |web_view, arg| {
                let data: serde_json::Value = serde_json::from_str(arg).unwrap();
                println!("{:?}", data);
                match data["type"].as_str().unwrap() {
                    "connect" => {
                        match Ipv4Addr::from_str(data["ip"].as_str().unwrap()) {
                            Ok(ip) => {tx.send(AppMessage::Connect(ip, data["port"].as_u64().unwrap() as u16)).ok();},
                            Err(_) => {web_view.eval(get_message_str("error", "Invalid IP.").as_str()).ok();}
                        };
                    },
                    "disconnect" => {tx.send(AppMessage::Disconnect).ok();},
                    "server" => {tx.send(AppMessage::Server(data["port"].as_u64().unwrap() as u16)).ok();},
                    "relieve" => {tx.send(AppMessage::RelieveControl).ok();},
                    "take" => {tx.send(AppMessage::TakeControl).ok();}
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
        let data = data.unwrap_or_default().to_string();
        let type_string = type_string.to_owned();
        handle.as_ref().unwrap().dispatch(move |webview| {
            webview.eval(get_message_str(type_string.as_str(), data.as_str()).as_str()).ok();
            Ok(())
        }).ok();
    }

    pub fn error(&mut self, error_string: &str) {
        self.invoke("error", Some(error_string));
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
}