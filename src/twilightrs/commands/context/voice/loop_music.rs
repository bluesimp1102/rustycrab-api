use std::error::Error;

use async_trait::async_trait;
use fluent_bundle::FluentArgs;
use twilight_model::gateway::payload::incoming::MessageCreate;

use crate::{
    twilightrs::{
        commands::context::{
            context_command::{ ContextCommand, GuildConfigModel },
            ParsedArg,
            ArgSpec,
            ArgType,
        },
        discord_client::{ DiscordClient, MessageContent },
        messages::DiscordEmbed,
        bot::voice_music::voice_manager::PlayerLoopState,
    },
    utilities::utils::ColorResolvables,
    cdn_avatar,
};
pub struct LoopMusicCommand {}

#[async_trait]
impl ContextCommand for LoopMusicCommand {
    fn name(&self) -> &'static str {
        "loop"
    }

    fn args(&self) -> Vec<ArgSpec> {
        vec![ArgSpec::new("type: current/one/all/queue", ArgType::Arg, true)]
    }

    async fn run(
        &self,
        client: DiscordClient,
        config: &GuildConfigModel,
        msg: &MessageCreate,
        command_args: Vec<ParsedArg>
    ) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
        let guild_id = msg.guild_id.ok_or(
            client.get_locale_string(&config.locale, "command-guildonly", None)
        )?;

        let (key, color) = if let Some(_) = client.voice_music_manager.songbird.get(guild_id) {
            if !client.is_user_in_same_channel_as_bot(guild_id, msg.author.id).await? {
                ("music-not-same-channel", ColorResolvables::Red)
            } else {
                let loop_type = if let Some(ParsedArg::Arg(state)) = command_args.first() {
                    state
                } else {
                    "queue"
                };

                match loop_type {
                    "current" | "song" | "track" | "one" | "1" => {
                        if
                            let Some(track_handle) = client.voice_music_manager
                                .get_play_queue(guild_id)
                                .current()
                        {
                            track_handle.enable_loop()?;
                            client.voice_music_manager.set_loop_state(
                                guild_id,
                                PlayerLoopState::LoopCurrentTrack
                            );
                            ("command-loop-track", ColorResolvables::Green)
                        } else {
                            ("command-loop-track-failed", ColorResolvables::Red)
                        }
                    }
                    "queue" | "all" => {
                        client.voice_music_manager.set_loop_state(
                            guild_id,
                            PlayerLoopState::LoopQueue
                        );

                        ("command-loop-queue", ColorResolvables::Green)
                    }
                    _ => { ("command-loop-invalid", ColorResolvables::Red) }
                }
            }
        } else {
            ("music-no-voice", ColorResolvables::Red)
        };

        client.reply_message(
            msg.channel_id,
            msg.id,
            MessageContent::DiscordEmbeds(
                vec![DiscordEmbed {
                    description: Some(client.get_locale_string(&config.locale, key, None)),
                    color: Some(color.as_u32()),
                    footer_text: Some(
                        client.get_locale_string(
                            &config.locale,
                            "requested-user",
                            Some(
                                &FluentArgs::from_iter(vec![("username", msg.author.name.clone())])
                            )
                        )
                    ),
                    footer_icon_url: msg.author.avatar.map(|hash| cdn_avatar!(msg.author.id, hash)),
                    ..Default::default()
                }]
            )
        ).await?;

        Ok(())
    }
}
