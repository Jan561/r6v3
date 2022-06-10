use crate::conf::ConfigKey;
use crate::movie::worker::{spawn_movie_worker, Message as WorkerMessage, WorkerChannel};
use crate::movie::{handle_groupwatch_default_channel, MOVIE_URIS};
use crate::sql::movie::uuid_from_vc;
use crate::sql::SqlKey;
use crate::voice::vc_is_empty;
use async_trait::async_trait;
use log::{debug, error, info};
use serenity::client::{Context, EventHandler};
use serenity::model::channel::Message;
use serenity::model::gateway::Activity;
use serenity::model::id::ChannelId;
use serenity::model::id::GuildId;
use serenity::model::voice::VoiceState;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Default)]
pub struct Handler {
    movie_worker_spawned: AtomicBool,
}

#[async_trait]
impl EventHandler for Handler {
    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        info!("Bot up and running!");

        let activity = Activity::playing("Powered by https://www.rust-lang.org/");

        ctx.set_activity(activity).await;

        let worker_running = self.movie_worker_spawned.swap(true, Ordering::Relaxed);

        if worker_running {
            return;
        }

        let ctx = Arc::new(ctx);

        let tx = spawn_movie_worker(Arc::clone(&ctx));

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
                conf.guilds.get_by_right(&guild.guild_id).and_then(|guild| {
                    conf.movie_time
                        .get(guild)
                        .map(|x| (x.text_channel, x.voice_channel))
                })
            };

            if let Some(channels) = mt_channel {
                if msg.channel_id == channels.0
                    && MOVIE_URIS.iter().any(|x| msg.content.starts_with(x))
                {
                    debug!(
                        "Received group watch link in default channel of guild {} from user {}#{}.",
                        guild.guild_id, msg.author.name, msg.author.discriminator
                    );

                    if let Err(why) =
                        handle_groupwatch_default_channel(&ctx, ChannelId(channels.1), &msg).await
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

        if new.channel_id.is_some() {
            handle_voice_join(&ctx, &new).await;
        }

        if old.is_some() {
            handle_voice_leave(&ctx, &old.unwrap()).await;
        }
    }
}

async fn handle_voice_join(ctx: &Context, new: &VoiceState) {
    let data = ctx.data.read().await;

    let mut sql = match data.get::<SqlKey>().unwrap().connection.get() {
        Ok(sql) => sql,
        Err(why) => {
            error!("Error getting SQL connection: {}", why);
            return;
        }
    };

    let uuid = match uuid_from_vc(&mut sql, new.channel_id.unwrap()) {
        Ok(uuid) => uuid,
        Err(why) => {
            error!("Error during SQL query: {}", why);
            return;
        }
    };

    if let Some(uuid) = uuid {
        let tx = data.get::<WorkerChannel>().unwrap();
        debug!("Member joined group watch, keeping it alive: {}", uuid);
        tri!(
            tx.send(WorkerMessage::KeepAlive(uuid)).await,
            "Receiver dropped message"
        );
    }
}

async fn handle_voice_leave(ctx: &Context, old: &VoiceState) {
    let data = ctx.data.read().await;
    let mut sql = match data.get::<SqlKey>().unwrap().connection.get() {
        Ok(sql) => sql,
        Err(why) => {
            error!("Error getting SQL connection: {}", why);
            return;
        }
    };

    if let Some(guild) = old.guild_id {
        let guild = match guild.to_guild_cached(&ctx) {
            Some(g) => g,
            None => return,
        };

        if !vc_is_empty(&guild, old.channel_id.unwrap()) {
            return;
        }

        let uuid = match uuid_from_vc(&mut sql, old.channel_id.unwrap()) {
            Ok(uuid) => uuid,
            Err(why) => {
                error!("Error during SQL query: {}", why);
                return;
            }
        };

        if let Some(uuid) = uuid {
            let tx = data.get::<WorkerChannel>().unwrap();
            debug!("Last group watch member left associated voice channel, starting inactivity countdown: {}.", uuid);
            tri!(
                tx.send(WorkerMessage::Inactive(uuid)).await,
                "Receiver dropped message"
            );
        }
    }
}
