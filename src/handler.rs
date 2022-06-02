use crate::conf::ConfigKey;
use crate::movie::{handle_groupwatch_default_channel, MOVIE_URIS};
use async_trait::async_trait;
use log::{error, info};
use serenity::client::{Context, EventHandler};
use serenity::model::channel::Message;
use serenity::model::gateway::Activity;
use serenity::model::id::GuildId;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        info!("Bot up and running!");

        let activity = Activity::playing("Powered by https://www.rust-lang.org/");

        ctx.set_activity(activity).await;
    }

    async fn message(&self, ctx: Context, msg: Message) {
        let mt_channel = {
            let data = ctx.data.read().await;
            let conf = data.get::<ConfigKey>().unwrap();
            conf.movie_time.as_ref().map(|x| x.text_channel)
        };

        if let Some(channel) = mt_channel {
            if msg.channel_id == channel && MOVIE_URIS.iter().any(|x| msg.content.starts_with(x)) {
                if let Err(why) = handle_groupwatch_default_channel(&ctx, &msg).await {
                    error!("Error processing group watch link: {}", why);
                }
            }
        }
    }
}
