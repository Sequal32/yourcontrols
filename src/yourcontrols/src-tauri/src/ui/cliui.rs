use std::{io::Write, str::FromStr, thread};

use crossbeam_channel::{unbounded, Receiver};

use super::{cmd::UiEvents, Ui};

#[derive(Debug, Clone)]
enum CliState {
    Prompt(Option<PromptState>),
    Waiting,
    Blocked,
}

#[derive(Debug, Clone)]
enum PromptState {
    // Joining a session
    JoinAddr,
    JoinPort(String),
    JoinSession,
    JoinName(JoinConnectionArgs),
    // Host
    HostPort,
    HostName(HostConnectionArgs),
}

#[derive(Debug, Clone)]
enum JoinConnectionArgs {
    SessionCode(String),
    Addr(String, u16),
}

#[derive(Debug, Clone)]
enum HostConnectionArgs {
    ServerHost,
    SelfHost(u16),
}

fn get_state_for(prompt: PromptState) -> CliState {
    CliState::Prompt(Some(prompt))
}

pub struct CliUi {
    state: CliState,
    input_rx: Receiver<String>,
}

impl CliUi {
    fn write_message(&self, msg: &str) {
        print!("{}", msg);
        std::io::stdout().flush();
    }

    fn parse_arg<T: FromStr>(&self, input: &str, msg: &str) -> Option<T> {
        match input.parse() {
            Ok(t) => Some(t),
            Err(_) => {
                self.write_message(msg);
                None
            }
        }
    }

    fn process_prompt(&mut self, input: &str, prompt: PromptState) -> Option<UiEvents> {
        let mut cmd: Option<UiEvents> = None;

        let new_state = match prompt.clone() {
            PromptState::JoinAddr => get_state_for(PromptState::JoinPort(input.to_string())),
            PromptState::JoinPort(addr) => self
                .parse_arg(input, "Invalid port.")
                .map(|port| {
                    get_state_for(PromptState::JoinName(JoinConnectionArgs::Addr(addr, port)))
                })
                .unwrap_or(get_state_for(prompt)),
            PromptState::JoinSession => get_state_for(PromptState::JoinName(
                JoinConnectionArgs::SessionCode(input.to_string()),
            )),
            PromptState::JoinName(args) => {
                let join_payload = match args {
                    JoinConnectionArgs::SessionCode(a) => UiEvents::Join {
                        port: None,
                        session_code: Some(a),
                        server_ip: None,
                        username: input.to_string(),
                    },
                    JoinConnectionArgs::Addr(ip, port) => UiEvents::Join {
                        port: Some(port),
                        session_code: None,
                        server_ip: Some(ip),
                        username: input.to_string(),
                    },
                };
                cmd = Some(join_payload);

                self.write_message("Attempting connection...");

                CliState::Blocked
            }
            PromptState::HostPort => self
                .parse_arg(input, "Invalid port.")
                .map(|port| {
                    get_state_for(PromptState::HostName(HostConnectionArgs::SelfHost(port)))
                })
                .unwrap_or(get_state_for(prompt)),
            PromptState::HostName(args) => {
                let host_payload = match args {
                    HostConnectionArgs::SelfHost(port) => UiEvents::Host {
                        port: Some(port),
                        username: input.to_string(),
                    },
                    HostConnectionArgs::ServerHost => UiEvents::Host {
                        port: None,
                        username: input.to_string(),
                    },
                };
                cmd = Some(host_payload);

                self.write_message("Starting server...");

                CliState::Blocked
            }
            _ => get_state_for(prompt),
        };

        self.set_state(new_state);

        cmd
    }

    fn set_state(&mut self, state: CliState) {
        self.state = state;

        match &self.state {
            CliState::Prompt(Some(p)) => match p {
                PromptState::JoinAddr => self.write_message("Peer IP address: "),
                PromptState::JoinPort(_) => self.write_message("Peer port: "),
                PromptState::JoinSession => self.write_message("Session Code: "),
                PromptState::JoinName(_) | PromptState::HostName(_) => self.write_message("Name: "),
                PromptState::HostPort => self.write_message("Port to host on: "),
            },
            CliState::Waiting => self.write_message("Enter a command: "),
            _ => {}
        }
    }

    fn process_wait(&mut self, input: &str) -> Option<UiEvents> {
        let mut cmd: Option<UiEvents> = None;

        let new_state = match input {
            "direct" => get_state_for(PromptState::JoinAddr),
            "p2p" => get_state_for(PromptState::JoinSession),
            "selfhost" => get_state_for(PromptState::HostPort),
            "serverhost" => get_state_for(PromptState::HostName(HostConnectionArgs::ServerHost)),
            _ => CliState::Waiting,
        };

        self.set_state(new_state);

        cmd
    }

    fn process_message(&mut self, input: &str) -> Option<UiEvents> {
        let input = input.trim();

        match &mut self.state {
            CliState::Prompt(prompt) => {
                if input == "q" {
                    self.set_state(CliState::Waiting);
                    return None;
                }

                let prompt = prompt.take().unwrap();
                self.process_prompt(input, prompt)
            }
            CliState::Waiting => self.process_wait(input),
            CliState::Blocked => None,
        }
    }
}

impl Ui for CliUi {
    fn run() -> Self {
        let (tx, rx) = unbounded();

        thread::spawn(move || loop {
            let mut buf = String::new();
            match std::io::stdin().read_line(&mut buf) {
                Ok(_) => tx.send(buf),
                Err(_) => todo!(),
            }
            .ok();
        });

        Self {
            state: CliState::Waiting,
            input_rx: rx,
        }
    }

    fn send_message(&mut self, event: UiEvents) {
        println!("{:?}", event);
        self.set_state(CliState::Waiting);
    }

    fn next_event(&mut self) -> Option<UiEvents> {
        match self.input_rx.try_recv() {
            Ok(input) => self.process_message(&input),
            Err(_) => None,
        }
    }
}
