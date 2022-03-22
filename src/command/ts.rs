use crate::command::tri;
use crate::permission::has_permission;
use crate::permission::rbac::RbacPermission;
use crate::sql::model::{Member, MemberBuilder};
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
        _ => Err(SimpleError::UsageError(format!("Unknown sub command {}.", sub)).into()),
    }
}

async fn ts_connect(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let sql = data.get::<SqlKey>().unwrap();
    let client_uuid = args.single::<String>().unwrap();

    let existing = Member::get(&sql, msg.author.id.0 as i64, false).await?;

    macro_rules! insert {
        () => {{
            MemberBuilder::default()
                .user_id(msg.author.id.0 as i64)
                .client_uuid(client_uuid)
                .insertion_pending(Some(true))
                .build()
                .insert(sql)
                .await
        }};
    }

    match existing {
        Some(mut m) => {
            if m.insertion_pending() {
                m.modify(|edit| edit.client_uuid(client_uuid)).await?;
            } else {
                m.modify(|edit| edit.removal_pending(Some(true))).await?;
                insert!()?;
            }
        }
        None => insert!().map(|_| ())?,
    };

    tri!(
        msg.reply(ctx, "Successfully connected your teamspeak client.")
            .await,
        "Error replying to author"
    );

    Ok(())
}

async fn ts_disconnect(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let sql = data.get::<SqlKey>().unwrap();

    let mut member = Member::get(sql, msg.author.id.0 as i64, false)
        .await?
        .ok_or_else(|| {
            SimpleError::UsageError(
                "There is currently no teamspeak client associated with your discord.".to_owned(),
            )
        })?;

    if member.insertion_pending() {
        member.destroy().await?;
    } else {
        member
            .modify(|edit| edit.removal_pending(Some(true)))
            .await?;
    }

    tri!(
        msg.reply(ctx, "Successfully removed the connection.").await,
        "Error replying to author"
    );
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
