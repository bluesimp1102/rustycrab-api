use std::error::Error;

use async_trait::async_trait;
use songbird::tracks::PlayMode;
use twilight_model::gateway::payload::incoming::MessageCreate;

use crate::{
    twilightrs::{
        commands::context::{ context_command::{ ContextCommand, GuildConfigModel }, ParsedArg },
        discord_client::DiscordClient,
        utils::send_response_message,
    },
    utilities::utils::ColorResolvables,
};
pub struct ResumeMusicCommand {}

#[async_trait]
impl ContextCommand for ResumeMusicCommand {
    fn name(&self) -> &'static str {
        "resume"
    }

    async fn run(
        &self,
        client: DiscordClient,
        config: &GuildConfigModel,
        msg: &MessageCreate,
        _: Vec<ParsedArg>
    ) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
        let guild_id = msg.guild_id.ok_or(
            client.get_locale_string(&config.locale, "command-guildonly", None)
        )?;

        let _ = client.fetch_call_lock(guild_id, Some(&config.locale)).await?;
        client.verify_same_voicechannel(guild_id, msg.author.id, Some(&config.locale)).await?;

        let handle = client.fetch_trackhandle(guild_id, Some(&config.locale)).await?;

        let info = handle.get_info().await?;

        let (key, color) = if info.playing == PlayMode::Pause {
            if let Ok(_) = handle.play() {
                ("command-resume-success", ColorResolvables::Red)
            } else {
                ("command-resume-failed", ColorResolvables::Red)
            }
        } else {
            ("command-resume-notpaused", ColorResolvables::Red)
        };

        send_response_message(&client, config, msg, key, color).await?;
        Ok(())
    }
}
