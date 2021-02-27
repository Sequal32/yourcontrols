use std::{thread::{sleep, spawn}, time::Duration};
use crossbeam_channel::{Sender, unbounded};
mod cmd;
mod util;

pub use util::*;


pub struct Ui {
    tx: Sender<UIEvents>
}

impl Ui {
    pub fn run() -> Self {
        let (tx, rx) = unbounded::<UIEvents>();

        spawn(move || {

            let app = tauri::AppBuilder::new()
            .invoke_handler(|_webview, arg| {
                use cmd::Cmd::*;
                // match serde_json::from_str(arg) {
                //     Err(e) => {
                //         Err(e.to_string())
                //     }
                //     Ok(command) => {
                //         match command {
                            
                //         }
                //         Ok(())
                //     }
                // }
                todo!()
            })
            .setup(move |_webview, _| {

                let rx = rx.clone();
                let mut webview = _webview.as_mut();

                spawn(move || {
                    
                    loop {
                        match rx.try_recv() {
                            Ok(event) => match event {}
                            Err(_) => {}
                        }

                        sleep(Duration::from_millis(10));
                    }
                    

                });

            })
            .build()
            .run();
            

        });

        Self {
            tx
        }

    }
}
