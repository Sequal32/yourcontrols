mod events;

use base64::{decode, encode};
use discord_game_sdk::{Activity, Discord};
use events::YourControlsDiscordEvents;
use std::{net::SocketAddr, str::FromStr};

type Secret = String;

pub enum JoinMethod {
  SessionCode(String),
  Ip(SocketAddr),
}

pub enum DiscordEvent {
  Join { method: JoinMethod },
  Invited { secret: Secret },
}

// Write unit tests for this struct
struct SecretEncoder {
  // Your choice of what to write here
}

impl SecretEncoder {
  fn encode_ip(addr: SocketAddr) -> Secret {
    encode(addr.to_string().as_bytes())
  }

  fn encode_session_id(session_id: String) -> Secret {
    encode(session_id.as_bytes())
  }

  fn decode_secret(secret: &str) -> JoinMethod {
    let mut result = String::from_utf8(decode(secret).unwrap()).unwrap();
    match SocketAddr::from_str(&result) {
      Ok(addr) => JoinMethod::Ip(addr),
      Err(error) => JoinMethod::SessionCode(result),
    }
  }
}

pub struct YourControlsDiscord<'a> {
  discord: Discord<'a, YourControlsDiscordEvents>,
  activity: Activity,
}

impl<'a> YourControlsDiscord<'a> {
  pub fn new(client_id: i64) -> Self {
    Self {
      discord: Discord::<YourControlsDiscordEvents>::new(client_id).unwrap(),
      activity: Activity::empty(),
    }
  }
  pub fn do_callbacks(&mut self) {
    self.discord.run_callbacks();
  }
  pub fn get_pending_events(&mut self) -> Option<DiscordEvent> {
    match self.discord.event_handler().as_ref().unwrap().rx.try_recv() {
      Ok(msg) => Some(msg),
      Err(_) => None,
    }
  }
  pub fn set_activity(
    &mut self,
    with_state: &str,
    with_details: &str,
    connected_count: u32,
    start_time: i64,
  ) {
    self.activity.with_state(with_state);
    self.activity.with_details(with_details);
    self.activity.with_party_amount(connected_count);
    self.activity.with_party_capacity(3);
    self.activity.with_large_image_key("icon");
    self.activity.with_start_time(start_time);
  }
  pub fn update_activity(&mut self) {
    self
      .discord
      .update_activity(&self.activity, |_, e| println!("{:?}", e))
  }
}
