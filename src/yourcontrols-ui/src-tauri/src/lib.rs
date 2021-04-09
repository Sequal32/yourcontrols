use cmd::Cmd;
use cmd::UIEvents;
use crossbeam_channel::{unbounded, Receiver, Sender};
use log::error;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, TcpStream};
use std::option::Option;
use std::thread::{sleep, spawn};
use std::time::Duration;
use tungstenite::{self, HandshakeError, Message, WebSocket};
use yourcontrols_types::Error;

pub mod cmd;
pub mod util;

pub use cmd::*;
pub use util::*;

macro_rules! events {
    ($V:ident) => {
        $V!(
            // CMD Variant, Javascript receive string
            (StartUpText, startUpText),
            (InitData, initData),
            (LoadingComplete, loadingComplete),
            (NetworkTestResult, networkTestResult),
        )
    };
}

pub struct Ui {
    listener: TcpListener,
    active_stream: Option<WebSocket<TcpStream>>,
    sent_connected: bool,
    ui_tx: Sender<UIEvents>,
    app_rx: Receiver<Cmd>,
}

impl Ui {
    pub fn run() -> Self {
        let listener = TcpListener::bind(SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::new(127, 0, 0, 1),
            40121,
        )))
        .unwrap();
        listener.set_nonblocking(true).ok();

        let (ui_tx, ui_rx) = unbounded::<UIEvents>();
        let (app_tx, app_rx) = unbounded::<Cmd>();
        spawn(move || {
            let _app = tauri::AppBuilder::new()
                .invoke_handler(move |_webview, arg| match serde_json::from_str(arg) {
                    Err(e) => Err(e.to_string()),
                    Ok(command) => {
                        app_tx.send(command).ok();
                        Ok(())
                    }
                })
                .setup(move |_webview, _| {
                    let ui_rx = ui_rx.clone();
                    let mut webview = _webview.as_mut();
                    spawn(move || loop {

                        macro_rules! process_event {
                            ($( ($event_name: ident, $cmd_name: expr), )*) => {
                                let result = match ui_rx.try_recv() {
                                    Ok(event) => match &event {
                                        $(
                                            UIEvents::$event_name { .. } => {
                                                tauri::event::emit(&mut webview, "$cmd_name", Some(event))
                                            }
                                        )*
                                    }
                                    _ => continue
                                };

                                if let Err(e) = result {
                                    error!(target: "yourcontrols-ui", "Could not emit event to tauri: {:?}", e)
                                }
                            }
                        }

                        events!(process_event);

                        sleep(Duration::from_millis(10));
                    });
                })
                .build()
                .run();
        });
        Self {
            ui_tx,
            app_rx,
            listener,
            active_stream: None,
            sent_connected: true,
        }
    }

    pub fn send_message_app(&self, event: UIEvents) {
        self.ui_tx.send(event).ok();
    }

    pub fn send_message_game_ui(&mut self, payload: GameUiPayloads) -> Result<(), Error> {
        if let Some(active_stream) = self.active_stream.as_mut() {
            active_stream.write_message(Message::Text(serde_json::to_string(&payload)?))?
        }

        Ok(())
    }

    fn accept_connections(&mut self) {
        if let Ok((stream, _)) = self.listener.accept() {
            let mut handshake_result = tungstenite::accept(stream);
            loop {
                match handshake_result {
                    Ok(ws) => {
                        self.sent_connected = false;
                        self.active_stream = Some(ws);
                        break;
                    }
                    Err(HandshakeError::Interrupted(hs)) => {
                        handshake_result = hs.handshake();
                    }
                    _ => break,
                }

                sleep(Duration::from_millis(1))
            }
        }
    }

    pub fn get_pending_events_game_ui(&mut self) -> Option<GameUiPayloads> {
        self.accept_connections();
        if let Some(ws) = &mut self.active_stream {
            if !self.sent_connected {
                self.sent_connected = true;
                return Some(GameUiPayloads::Connected);
            }

            return match ws.read_message() {
                Ok(Message::Text(text)) => match serde_json::from_str(&text) {
                    Ok(payload) => Some(payload),
                    Err(e) => {
                        error!("{} {:?}", text, e);
                        None
                    }
                },
                Err(tungstenite::Error::ConnectionClosed) => Some(GameUiPayloads::Disconnected),
                _ => None,
            };
        }

        return None;
    }
    pub fn get_pending_events_app(&self) -> Option<Cmd> {
        self.app_rx.try_recv().ok()
    }
}
