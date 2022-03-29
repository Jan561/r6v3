use crate::command::{tri, usage_error, CMD_PREFIX};
use crate::conf::ConfigKey;
use crate::permission::has_permission;
use crate::permission::rbac::RbacPermission;
use crate::sql::model::{NewTsMember, TsMember};
use crate::sql::SqlKey;
use diesel::Connection;
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
    let config = data.get::<ConfigKey>().unwrap();
    // Error type: r2d2::Error
    let mut sql = data.get::<SqlKey>().unwrap().connection.get()?;
    let c_uuid = args
        .single::<String>()
        .map_err(|_| usage_error!("Syntax: {}ts connect <Client ID>.", CMD_PREFIX))?;

    sql.transaction(|c| {
        config
            .servers
            .iter()
            .filter(|(_, v)| v.ts.is_some())
            .try_for_each(|(k, _)| {
                let id = msg.author.id.0 as i64;
                TsMember::schedule_deletion(id, k, c)?;
                NewTsMember {
                    user_id: id,
                    client_uuid: &c_uuid,
                    instance: k,
                }
                .insert(c)
                .and_then(|success| {
                    if !success {
                        Err(usage_error!("Client ID already taken."))
                    } else {
                        Ok(())
                    }
                })
            })
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
    let config = data.get::<ConfigKey>().unwrap();
    let mut sql = data.get::<SqlKey>().unwrap().connection.get()?;
    let id = msg.author.id.0 as i64;

    let res = sql.transaction(|c| {
        config
            .servers
            .iter()
            .filter(|(_, v)| v.ts.is_some())
            .try_fold(false, |res, (k, _)| {
                TsMember::schedule_deletion(id, k, c).map(|x| res || x)
            })
    })?;

    if res {
        tri!(
            msg.reply(ctx, "Successfully removed the connection.").await,
            "Error replying to author"
        );
    } else {
        return Err(usage_error!("There is no connection associated with your account.").into());
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
