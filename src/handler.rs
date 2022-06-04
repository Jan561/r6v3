use crate::conf::ConfigKey;
use crate::movie::worker::{spawn_movie_worker, Message as WorkerMessage, WorkerChannel};
use crate::movie::{handle_groupwatch_default_channel, MOVIE_URIS};
use crate::sql::movie::uuid_from_vc;
use crate::sql::SqlKey;
use async_trait::async_trait;
use log::{error, info};
use serenity::client::{Context, EventHandler};
use serenity::model::channel::Message;
use serenity::model::gateway::Activity;
use serenity::model::id::ChannelId;
use serenity::model::id::GuildId;
use serenity::model::voice::VoiceState;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        info!("Bot up and running!");

        let activity = Activity::playing("Powered by https://www.rust-lang.org/");

        ctx.set_activity(activity).await;

        let tx = spawn_movie_worker(ctx.clone());

        {
            let mut data = ctx.data.write().await;
            data.insert::<WorkerChannel>(tx);
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {
        let channel = match msg.channel(&ctx).await {
            Ok(c) => c,
            Err(why) => {
                error!("Error getting channel information: {}", why);
                return;
            }
        };

        if let Some(guild) = channel.guild() {
            let mt_channel = {
                let data = ctx.data.read().await;
                let conf = data.get::<ConfigKey>().unwrap();
                conf.guilds
                    .get_by_right(&guild.guild_id)
                    .and_then(|guild| conf.movie_time.get(guild).map(|x| x.text_channel))
            };

            if let Some(channel) = mt_channel {
                if msg.channel_id == channel
                    && MOVIE_URIS.iter().any(|x| msg.content.starts_with(x))
                {
                    if let Err(why) =
                        handle_groupwatch_default_channel(&ctx, ChannelId(channel), &msg).await
                    {
                        error!("Error processing group watch link: {}", why);
                    }
                }
            }
        }
    }

    async fn voice_state_update(&self, ctx: Context, old: Option<VoiceState>, new: VoiceState) {
        if old
            .as_ref()
            .map_or(false, |x| x.channel_id == new.channel_id)
        {
            return;
        }

        let data = ctx.data.read().await;
        let mut sql = match data.get::<SqlKey>().unwrap().connection.get() {
            Ok(sql) => sql,
            Err(why) => {
                error!("Error getting SQL connection: {}", why);
                return;
            }
        };

        if let Some(id) = new.channel_id {
            let uuid = match uuid_from_vc(&mut sql, id) {
                Ok(uuid) => uuid,
                Err(why) => {
                    error!("Error during SQL query: {}", why);
                    return;
                }
            };

            if let Some(uuid) = uuid {
                let tx = data.get::<WorkerChannel>().unwrap();

                tri!(
                    tx.send(WorkerMessage::KeepAlive(uuid)).await,
                    "Receiver dropped message"
                );
            }
        }

        if let Some(VoiceState {
            channel_id: Some(id),
            guild_id: Some(guild),
            ..
        }) = &old
        {
            let guild = match guild.to_guild_cached(&ctx) {
                Some(g) => g,
                None => return,
            };

            if guild
                .voice_states
                .values()
                .any(|x| matches!(x.channel_id, Some(c) if c == *id))
            {
                return;
            }

            let uuid = match uuid_from_vc(&mut sql, *id) {
                Ok(uuid) => uuid,
                Err(why) => {
                    error!("Error during SQL query: {}", why);
                    return;
                }
            };

            if let Some(uuid) = uuid {
                let tx = data.get::<WorkerChannel>().unwrap();

                tri!(
                    tx.send(WorkerMessage::KeepAlive(uuid)).await,
                    "Receiver dropped message"
                );
            }
        }
    }
}
