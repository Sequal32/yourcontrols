use crate::{DiscordEvent, SecretEncoder};
use crossbeam_channel::{Receiver, Sender};

use discord_game_sdk::{Discord, EventHandler};

pub struct YourControlsDiscordEvents {
  pub tx: Sender<DiscordEvent>,
  pub rx: Receiver<DiscordEvent>,
}

impl EventHandler for YourControlsDiscordEvents {
  fn on_activity_join(&mut self, _discord: &Discord<'_, Self>, secret: &str) {
    self.tx.send(DiscordEvent::Join {
      method: SecretEncoder::decode_secret(secret),
    });
  }

  fn on_activity_join_request(
    &mut self,
    _discord: &Discord<'_, Self>,
    user: &discord_game_sdk::User,
  ) {
    println!("{:?}", user.username())
  }

  fn on_activity_invite(
    &mut self,
    _discord: &Discord<'_, Self>,
    kind: discord_game_sdk::Action,
    user: &discord_game_sdk::User,
    activity: &discord_game_sdk::Activity,
  ) {
    println!("{:?} {:?} {:?}", kind, user, activity)
  }
}
