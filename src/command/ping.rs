use crate::permission::HasPermission;
use async_trait::async_trait;
use serenity::client::Context;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::channel::Message;
use serenity::model::id::UserId;

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}

pub struct PingPermission;

#[async_trait]
impl HasPermission<PingPermission> for UserId {
    async fn has_permission(&self, _: &Context, _: &PingPermission) -> bool {
        true
    }
}
