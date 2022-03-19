use crate::permission::has_permission;
use crate::permission::rbac::RbacPermission;
use serenity::client::Context;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::channel::Message;

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

has_permission! { PingPermission }
