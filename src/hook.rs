use crate::command::ping::PingPermission;
use crate::command::start::StartPermission;
use crate::command::stop::StopPermission;
use crate::permission::{DefaultPermission, HasPermission};
use serenity::client::Context;
use serenity::framework::standard::macros::hook;
use serenity::framework::standard::CommandError;
use serenity::model::channel::Message;

#[hook]
pub async fn before_hook(ctx: &Context, msg: &Message, cmd_name: &str) -> bool {
    if !has_permission(ctx, msg, cmd_name).await {
        return false;
    }

    true
}

#[hook]
pub async fn after_hook(
    _ctx: &Context,
    _msg: &Message,
    _cmd_name: &str,
    res: Result<(), CommandError>,
) {
    if let Err(why) = res {
        eprintln!("{}", why);
    }
}

async fn has_permission(ctx: &Context, msg: &Message, cmd_name: &str) -> bool {
    let user = msg.author.id;
    match cmd_name {
        "ping" => user.has_permission(ctx, &PingPermission).await,
        "start" => user.has_permission(ctx, &StartPermission).await,
        "stop" => user.has_permission(ctx, &StopPermission).await,
        _ => user.has_permission(ctx, &DefaultPermission).await,
    }
}
