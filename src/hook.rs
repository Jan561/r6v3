use crate::command::ping::PingPermission;
use crate::command::start::StartPermission;
use crate::command::stop::StopPermission;
use crate::command::ts::{ConnectPermission, DisconnectPermission};
use crate::permission::HasPermission;
use crate::{SimpleError, SimpleResult};
use log::{error, info, warn};
use serenity::client::Context;
use serenity::framework::standard::macros::hook;
use serenity::framework::standard::{Args, CommandError, Delimiter};
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

    macro_rules! check_roles {
        ($perm:expr) => {{
            match msg.guild(ctx).await {
                None => false,
                Some(g) => match g.member(ctx, user).await {
                    Ok(member) => {
                        let mut check = false;
                        for r in member.roles.iter() {
                            if r.has_permission(ctx, $perm).await {
                                check = true;
                                break;
                            }
                        }
                        check
                    }
                    Err(e) => return Err(e.into()),
                },
            }
        }};
    }

    macro_rules! check_permission {
        ($perm:expr) => {{
            let p = $perm;
            user.has_permission(ctx, &p).await || check_roles!(&p)
        }};
    }

    let r = match cmd_name {
        "ping" => check_permission!(PingPermission),
        "start" => check_permission!(StartPermission::from_message(msg)?),
        "stop" => check_permission!(StopPermission::from_message(msg)?),
        "ts" => {
            let mut args = Args::new(&msg.content, &[Delimiter::Single(' ')]);
            let subcmd = args.advance().single::<String>().unwrap();
            match &*subcmd {
                "connect" => check_permission!(ConnectPermission),
                "disconnect" => check_permission!(DisconnectPermission),
                _ => false,
            }
        }
        _ => false,
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

    error!("Command execution unsuccessful: {:?}", err);
    print_error(err, ctx, msg).await;
}

async fn print_error(err: &SimpleError, ctx: &Context, msg: &Message) {
    let res = msg
        .reply(ctx, format!("An internal error occurred: {}", err))
        .await;

    if let Err(why) = res {
        warn!("An error occurred replying to the author.: {:?}", why);
    }
}
