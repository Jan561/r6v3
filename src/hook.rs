use crate::command::ping::PingPermission;
use crate::command::start::StartPermission;
use crate::command::stop::StopPermission;
use crate::permission::{DefaultPermission, HasPermission};
use log::{error, info};
use serenity::client::Context;
use serenity::framework::standard::macros::hook;
use serenity::framework::standard::CommandError;
use serenity::model::channel::Message;

#[hook]
pub async fn before_hook(ctx: &Context, msg: &Message, cmd_name: &str) -> bool {
    info!(
        "Received command {} from user {}#{}.",
        cmd_name, msg.author.name, msg.author.discriminator
    );

    if !has_permission(ctx, msg, cmd_name).await {
        info!(
            "Unauthorized command usage: {} from {}#{}.",
            cmd_name, msg.author.name, msg.author.discriminator
        );

        let res = msg.reply(ctx, "Not authorized.").await;
        if let Err(why) = res {
            error!("An error occurred replying to the author.: {:?}", why);
        }

        return false;
    }

    true
}

#[hook]
pub async fn after_hook(
    ctx: &Context,
    msg: &Message,
    cmd_name: &str,
    res: Result<(), CommandError>,
) {
    if let Err(why) = res {
        error!("{:?}", why);
        let res = msg
            .reply(ctx, format!("An error occurred on our end: {}", why))
            .await;

        if let Err(why) = res {
            error!("An error occurred replying to the author.: {:?}", why);
        }
    } else {
        info!(
            "Successfully processed {} of user {}#{}.",
            cmd_name, msg.author.name, msg.author.discriminator
        );
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
