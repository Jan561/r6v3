use crate::command::ping::PingPermission;
use crate::command::start::StartPermission;
use crate::command::stop::StopPermission;
use crate::permission::{DefaultPermission, HasPermission};
use crate::{SimpleError, SimpleResult};
use log::{error, info, warn};
use serenity::client::Context;
use serenity::framework::standard::macros::hook;
use serenity::framework::standard::CommandError;
use serenity::model::channel::Message;

#[hook]
pub async fn before_hook(ctx: &Context, msg: &Message, cmd_name: &str) -> bool {
    info!(
        "Received command {} from user {}#{} ({}).",
        cmd_name, msg.author.name, msg.author.discriminator, msg.author.id
    );

    let perm_check = has_permission(ctx, msg, cmd_name).await;

    match perm_check {
        Ok(false) => {
            info!(
                "Unauthorized command usage: {} from {}#{} ({}).",
                cmd_name, msg.author.name, msg.author.discriminator, msg.author.id
            );

            let res = msg.reply(ctx, "Not authorized.").await;
            if let Err(why) = res {
                warn!("An error occurred replying to the author.: {:?}", why);
            }

            false
        }

        Ok(true) => true,
        Err(why) => {
            handle_error(&why, ctx, msg).await;
            false
        }
    }
}

#[hook]
pub async fn after_hook(
    ctx: &Context,
    msg: &Message,
    cmd_name: &str,
    res: Result<(), CommandError>,
) {
    if let Err(why) = res {
        handle_error(why.downcast_ref::<SimpleError>().unwrap(), ctx, msg).await;
    } else {
        info!(
            "Successfully processed command {} from user {}#{} ({}).",
            cmd_name, msg.author.name, msg.author.discriminator, msg.author.id
        );
    }
}

async fn has_permission(ctx: &Context, msg: &Message, cmd_name: &str) -> SimpleResult<bool> {
    let user = msg.author.id;
    let r = match cmd_name {
        "ping" => user.has_permission(ctx, &PingPermission).await,
        "start" => {
            user.has_permission(ctx, &StartPermission::from_message(msg)?)
                .await
        }
        "stop" => {
            user.has_permission(ctx, &StopPermission::from_message(msg)?)
                .await
        }
        _ => user.has_permission(ctx, &DefaultPermission).await,
    };

    Ok(r)
}

async fn handle_error(err: &SimpleError, ctx: &Context, msg: &Message) {
    if let SimpleError::UsageError(ref why) = err {
        info!("Command usage error: {}", why);
        if let Err(inner) = msg.reply(ctx, why).await.map_err(Into::into) {
            print_error(&inner, ctx, msg).await;
        }

        return;
    }

    print_error(err, ctx, msg).await;
}

async fn print_error(err: &SimpleError, ctx: &Context, msg: &Message) {
    error!("Command execution unsuccessful: {:?}", err);
    let res = msg
        .reply(ctx, format!("An internal error occurred: {}", err))
        .await;

    if let Err(why) = res {
        warn!("An error occurred replying to the author.: {:?}", why);
    }
}
