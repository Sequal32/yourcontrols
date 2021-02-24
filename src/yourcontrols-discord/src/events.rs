use crate::{DiscordEvent, SecretEncoder};
use crossbeam_channel::{unbounded, Receiver, Sender};

use discord_game_sdk::{Discord, EventHandler};

pub struct YourControlsDiscordEvents {
    tx: Sender<DiscordEvent>,
    rx: Receiver<DiscordEvent>,
    secret: String,
}

impl YourControlsDiscordEvents {
    pub fn new() -> Self {
        let (tx, rx) = unbounded();
        Self {
            tx,
            rx,
            secret: String::new(),
        }
    }

    pub fn set_secret(&mut self, secret: String) {
        self.secret = secret;
    }

    pub fn get_receiver(&self) -> &Receiver<DiscordEvent> {
        &self.rx
    }
}

impl EventHandler for YourControlsDiscordEvents {
    fn on_activity_join(&mut self, _discord: &Discord<'_, Self>, secret: &str) {
        let secret = match SecretEncoder::decode_secret(secret) {
            Ok(secret) => secret,
            Err(_) => return,
        };

        self.tx.send(DiscordEvent::Join { method: secret }).ok();
    }

    fn on_activity_join_request(
        &mut self,
        discord: &Discord<'_, Self>,
        user: &discord_game_sdk::User,
    ) {
        discord.accept_invite(user.id(), |_discord, Result| {
            println!("on_activity_join_request {:?}", Result.unwrap())
        });
    }

    fn on_activity_invite(
        &mut self,
        discord: &Discord<'_, Self>,
        kind: discord_game_sdk::Action,
        user: &discord_game_sdk::User,
        activity: &discord_game_sdk::Activity,
    ) {

        self.tx.send(DiscordEvent::Invited {
            secret: activity.party_id().to_string(),
        }).ok();
    }

}
