use crate::permission::rbac::{HasRbacPermission, RbacPermission};
use crate::permission::HasPermission;
use crate::RbacKey;
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

impl RbacPermission for PingPermission {
    type T = &'static str;

    fn rbac(&self) -> &'static str {
        "/ping"
    }
}

#[async_trait]
impl HasPermission<PingPermission> for UserId {
    async fn has_permission(&self, ctx: &Context, p: &PingPermission) -> bool {
        let data = ctx.data.read().await;
        let rbac = data.get::<RbacKey>().unwrap();
        <Self as HasRbacPermission>::has_permission(self, p, rbac)
    }
}
