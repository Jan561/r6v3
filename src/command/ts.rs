use crate::permission::has_permission;
use crate::permission::rbac::RbacPermission;
use crate::sql::model::MemberBuilder;
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
        _ => Err(SimpleError::UsageError(format!("Unknown sub command {}.", sub)).into()),
    }
}

async fn ts_connect(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let sql = data.get::<SqlKey>().unwrap();
    let client_uuid = args.single::<String>().unwrap();
    MemberBuilder::default()
        .user_id(msg.author.id.0 as i64)
        .client_uuid(client_uuid)
        .build()
        .insert(sql)
        .await?;

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
