use cmd::Cmd;
use cmd::UIEvents;
use crossbeam_channel::{Receiver, Sender, unbounded};
use std::{option::Option, thread::{sleep, spawn}, time::Duration};
use log::{error};
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
                        match ui_rx.try_recv() {
                            Ok(event) => match &event {
                                UIEvents::StartUpText { .. } => {
                                    match tauri::event::emit(
                                        &mut webview,
                                        "startUpText",
                                        Some(event.clone()),
                                    ) {
                                        Ok(_) => {}
                                        Err(e) => {
                                            error!(target:"yourcontrols-ui", "{:?}", e)
                                        }
                                    };
                                }
                                UIEvents::InitData { .. } => {
                                    match tauri::event::emit(
                                        &mut webview,
                                        "initData",
                                        Some(event.clone()),
                                    ) {
                                        Ok(_) => {}
                                        Err(e) => {
                                            error!(target:"yourcontrols-ui", "{:?}", e)
                                        }
                                    };
                                }
                                UIEvents::LoadingComplete => {
                                    match tauri::event::emit(
                                        &mut webview,
                                        "loadingComplete",
                                        Some(event.clone()),
                                    ) {
                                        Ok(_) => {}
                                        Err(e) => {
                                            error!(target:"yourcontrols-ui", "{:?}", e)
                                        }
                                    };
                                }
                                UIEvents::NetworkTestResult { .. } => {
                                    match tauri::event::emit(
                                        &mut webview,
                                        "networkTestResult",
                                        Some(event.clone()),
                                    ) {
                                        Ok(_) => {}
                                        Err(e) => {
                                            error!(target:"yourcontrols-ui", "{:?}", e)
                                        }
                                    };
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

    pub fn get_pending_events(&self) -> Option<Cmd> {
        self.app_rx.try_recv().ok()
    }
}
