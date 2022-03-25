use crate::command::{tri, usage_error, CMD_PREFIX};
use crate::permission::has_permission;
use crate::permission::rbac::RbacPermission;
use crate::sql::model::TsMember;
use crate::sql::SqlKey;
use crate::SimpleError;
use serenity::client::Context;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::channel::Message;

#[command]
async fn ts(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let sub = args.quoted().single::<String>().unwrap();

    match &*sub {
        "connect" => ts_connect(ctx, msg, args).await,
        "disconnect" => ts_disconnect(ctx, msg).await,
        _ => Err(usage_error!("Unknown sub command {}.", sub).into()),
    }
}

async fn ts_connect(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    // Error type: r2d2::Error
    let mut sql = data.get::<SqlKey>().unwrap().connection.get()?;
    let c_uuid = args
        .single::<String>()
        .map_err(|_| usage_error!("Syntax: {}ts connect <Client ID>.", CMD_PREFIX))?;

    let member = TsMember {
        user_id: msg.author.id.0 as i64,
        client_uuid: c_uuid,
        insertion_pending: true,
        removal_pending: false,
    };

    TsMember::schedule_deletion(member.user_id, &mut sql)?;

    // Perform the insert
    member.insert(&mut sql).and_then(|success| {
        if !success {
            Err(SimpleError::UsageError(
                "Client ID already taken.".to_owned(),
            ))
        } else {
            Ok(())
        }
    })?;

    tri!(
        msg.reply(ctx, "Successfully connected your teamspeak client.")
            .await,
        "Error replying to author"
    );

    Ok(())
}

async fn ts_disconnect(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let mut sql = data.get::<SqlKey>().unwrap().connection.get()?;
    let id = msg.author.id.0 as i64;

    if TsMember::schedule_deletion(id, &mut sql)? {
        tri!(
            msg.reply(ctx, "Successfully removed the connection.").await,
            "Error replying to author"
        );
    } else {
        return Err(usage_error!("There is no connection associated with your account.",).into());
    }

    Ok(())
}

pub struct ConnectPermission;

impl RbacPermission for ConnectPermission {
    type T = &'static str;

    fn rbac(&self) -> &'static str {
        "/ts/connect"
    }
}

has_permission! { ConnectPermission }

pub struct DisconnectPermission;

impl RbacPermission for DisconnectPermission {
    type T = &'static str;

    fn rbac(&self) -> &'static str {
        "/ts/disconnect"
    }
}

has_permission! { DisconnectPermission }
