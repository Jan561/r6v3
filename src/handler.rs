use async_trait::async_trait;
use log::info;
use serenity::client::{Context, EventHandler};
use serenity::model::id::GuildId;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn cache_ready(&self, _ctx: Context, _guilds: Vec<GuildId>) {
        info!("Bot up and running!");
    }
}
