mod events;

use base64::{decode, encode};
use discord_game_sdk::{Activity, Discord, UserID};
use events::YourControlsDiscordEvents;
use std::{net::SocketAddr, str::FromStr, time::SystemTime};

use yourcontrols_types::Error;

type Secret = String;

pub enum JoinMethod {
    SessionCode(String),
    Ip(SocketAddr),
}

pub enum DiscordEvent {
    Join { method: JoinMethod },
    AskedToJoin { user_id: UserID },
    Invited { secret: Secret },
}

pub struct SecretEncoder {}

impl SecretEncoder {
    pub fn encode_ip(addr: SocketAddr) -> Secret {
        encode(addr.to_string().as_bytes())
    }

    pub fn encode_session_id(session_id: String) -> Secret {
        encode(session_id.as_bytes())
    }

    pub fn decode_secret(secret: &str) -> Result<JoinMethod, Error> {
        let decode_bytes = decode(secret)?;
        let decode_s = String::from_utf8(decode_bytes)?;

        Ok(match SocketAddr::from_str(&decode_s) {
            Ok(addr) => JoinMethod::Ip(addr),
            Err(_) => JoinMethod::SessionCode(decode_s),
        })
    }
}

pub struct YourControlsDiscord<'a> {
    discord: Discord<'a, YourControlsDiscordEvents>,
    activity: Activity,
}

impl<'a> YourControlsDiscord<'a> {
    pub fn new(client_id: i64) -> Self {
        let mut discord = Discord::<YourControlsDiscordEvents>::new(client_id).unwrap();

        *discord.event_handler_mut() = Some(YourControlsDiscordEvents::new());

        Self {
            discord,
            activity: Activity::empty(),
        }
    }
    pub fn do_callbacks(&mut self) {
        self.discord.run_callbacks().ok();
    }
    pub fn get_pending_events(&mut self) -> Option<DiscordEvent> {
        match self
            .discord
            .event_handler()
            .as_ref()
            .unwrap()
            .get_receiver()
            .try_recv()
        {
            Ok(msg) => Some(msg),
            Err(_) => None,
        }
    }
    pub fn update_activity(&mut self) {
        self.discord.update_activity(&self.activity, |_, _| {})
    }

    pub fn accept_invite(&mut self, user_id: UserID) {
        self.discord.accept_invite(user_id, |_, _| {})
    }

    pub fn set_large_image_key(&mut self, img_key: &str) {
        self.activity.with_large_image_key(img_key);
    }

    pub fn set_large_image_desc(&mut self, desc: &str) {
        self.activity.with_large_image_tooltip(desc);
    }

    pub fn set_lobby_info(&mut self, id: &str, ammount: u32, capacity: u32) {
        self.activity.with_party_id(id);
        self.activity.with_party_amount(ammount);
        self.activity.with_party_capacity(capacity);
    }

    pub fn set_secret(&mut self, secret: &str) {
        self.activity.with_join_secret(secret);
    }

    pub fn set_state(&mut self, state: &str) {
        self.activity.with_state(state);
    }

    pub fn set_details(&mut self, details: &str) {
        self.activity.with_details(details);
    }

    pub fn set_current_time(&mut self) {
        self.activity.with_start_time(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        );
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_bad_secret() {
        assert!(SecretEncoder::decode_secret("test").is_err());
    }

    #[test]
    fn test_encode_decode() {
        let encoded = SecretEncoder::encode_session_id("test".to_string());
        let _output = JoinMethod::SessionCode("test".to_string());
        assert!(std::matches!(
            SecretEncoder::decode_secret(&encoded),
            Ok(_output)
        ));
    }

    #[test]
    fn test_encode_decode_ip() {
        let addr: SocketAddr = "127.0.0.1:23213".parse().unwrap();
        let encoded = SecretEncoder::encode_ip(addr);
        let _output = JoinMethod::Ip(addr);
        assert!(std::matches!(
            SecretEncoder::decode_secret(&encoded),
            Ok(_output)
        ));
    }
}
