use cmd::Cmd;
use cmd::UIEvents;
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::{
    thread::{sleep, spawn},
    time::Duration,
};
pub mod cmd;
pub mod util;

pub use cmd::*;
pub use util::*;

pub struct Ui {
    ui_tx: Sender<UIEvents>,
    app_rx: Receiver<Cmd>,
}

impl Ui {
    pub fn run() -> Self {
        let (ui_tx, ui_rx) = unbounded::<UIEvents>();
        let (app_tx, app_rx) = unbounded::<Cmd>();
        spawn(move || {
            let app = tauri::AppBuilder::new()
                .invoke_handler(move |_webview, arg| match serde_json::from_str(arg) {
                    Err(e) => Err(e.to_string()),
                    Ok(command) => {
                        app_tx.send(command);
                        Ok(())
                    }
                })
                .setup(move |_webview, _| {
                    let ui_rx = ui_rx.clone();
                    let mut webview = _webview.as_mut();
                    spawn(move || loop {
                        match ui_rx.try_recv() {
                            Ok(event) => match &event {
                                UIEvents::StartUpText { .. } => {
                                    tauri::event::emit(
                                        &mut webview,
                                        "startUpText",
                                        Some(event.clone()),
                                    );
                                }
                                UIEvents::InitData { .. } => {
                                    tauri::event::emit(
                                        &mut webview,
                                        "initData",
                                        Some(event.clone()),
                                    );
                                }
                                UIEvents::LoadingComplete => {
                                    tauri::event::emit(
                                        &mut webview,
                                        "loadingComplete",
                                        Some(event.clone()),
                                    );
                                }
                                UIEvents::NetworkTestResult { .. } => {
                                    tauri::event::emit(
                                        &mut webview,
                                        "networkTestResult",
                                        Some(event.clone()),
                                    );
                                }
                            },
                            Err(_) => {}
                        }
                        sleep(Duration::from_millis(10));
                    });
                })
                .build()
                .run();
        });
        Self {
            ui_tx,
            app_rx,
        }
    }

    pub fn send_message(&self, event: UIEvents) {
        self.ui_tx.send(event).ok();
    }

    pub fn get_pending_events(&mut self) {
        self.app_rx.try_recv();
    }
}
