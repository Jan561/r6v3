use crate::command::ping::PingPermission;
use crate::permission::{DefaultPermission, HasPermission};
use serenity::client::Context;
use serenity::model::channel::Message;

pub async fn before_hook(ctx: &Context, msg: &Message, cmd_name: &str) -> bool {
    if !has_permission(ctx, msg, cmd_name).await {
        return false;
    }

    true
}

async fn has_permission(ctx: &Context, msg: &Message, cmd_name: &str) -> bool {
    let user = msg.author.id;
    match cmd_name {
        "ping" => user.has_permission(ctx, &PingPermission).await,
        _ => user.has_permission(ctx, &DefaultPermission).await,
    }
}
