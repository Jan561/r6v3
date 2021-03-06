pub mod worker;

use crate::movie::worker::{Message as WorkerMessage, WorkerChannel};
use crate::sql::movie::NewMovieChannel;
use crate::sql::uuid::Uuid;
use crate::sql::SqlKey;
use crate::voice::vc_is_empty;
use crate::SimpleResult;
use chrono::Utc;
use diesel::prelude::*;
use log::{debug, warn};
use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::model::id::{ChannelId, MessageId, UserId};
use serenity::model::mention::Mention;

pub const MOVIE_URIS: [&str; 1] = ["https://www.disneyplus.com/groupwatch/"];

pub fn groupwatch_create_msg(uri: impl AsRef<str>, creator: UserId) -> String {
    return format!(
        "GroupWatch: {}\n\nCreated by: {}",
        uri.as_ref(),
        Mention::from(creator),
    );
}

pub async fn handle_groupwatch_default_channel(
    ctx: &Context,
    vc: ChannelId,
    msg: &Message,
) -> SimpleResult<()> {
    let data = ctx.data.read().await;
    let mut sql = data.get::<SqlKey>().unwrap().connection.get()?;

    use crate::schema::movie_channels::dsl;
    let old_gw: Option<(Uuid, i64)> =
        diesel::delete(dsl::movie_channels.filter(dsl::vc.eq(vc.0 as i64)))
            .returning((dsl::id, dsl::bot_msg))
            .get_result(&mut sql)
            .optional()?;

    if let Some((uuid, old_msg)) = old_gw {
        tri!(
            msg.channel_id.delete_message(ctx, old_msg as u64).await,
            "Error deleting old message",
        );

        let tx = data.get::<WorkerChannel>().unwrap();
        tri!(
            tx.send(WorkerMessage::Delete(uuid)).await,
            "Receiver dropped channel message"
        );
    }

    msg.delete(ctx).await?;
    let new_msg = msg
        .channel(ctx)
        .await?
        .guild()
        .expect("The group watch default channel must be a guild channel.")
        .say(ctx, groupwatch_create_msg(&msg.content, msg.author.id))
        .await?;

    let new_movie_channel = NewMovieChannel {
        id: Uuid::random(),
        uri: &new_msg.content,
        vc: vc.0 as i64,
        bot_msg_channel_id: new_msg.channel_id.0 as i64,
        bot_msg: new_msg.id.0 as i64,
        guild: msg.guild_id.unwrap().0 as i64,
        creator: new_msg.author.id.0 as i64,
        created_at: Utc::now().naive_utc(),
    };

    let result = new_movie_channel.insert(&mut sql)?;

    match result {
        Some(uuid) => {
            let tx = data.get::<WorkerChannel>().unwrap();
            let guild = msg.guild(ctx).unwrap();

            debug!(
                "Successfully created group watch for {}#{} ({}) with uuid {}.",
                msg.author.name, msg.author.discriminator, msg.author.id, uuid
            );

            if vc_is_empty(&guild, vc) {
                debug!(
                    "New group watch currently empty, starting inactivity countdown: {}",
                    uuid
                );
                tri!(
                    tx.send(WorkerMessage::Inactive(uuid)).await,
                    "Receiver dropped channel message"
                );
            }
        }
        None => {
            warn!(
                "Failed to insert new group watch entry for default channel into DB, deleting it."
            );
            tri!(
                new_msg.delete(ctx).await,
                "Error deleting group watch message"
            );
        }
    }

    Ok(())
}

pub async fn delete_channel(context: &Context, uuid: Uuid) -> SimpleResult<bool> {
    let data = context.data.read().await;
    let mut sql = data.get::<SqlKey>().unwrap().connection.get()?;
    use crate::schema::movie_channels::dsl;

    let res = diesel::delete(dsl::movie_channels.filter(dsl::id.eq(uuid)))
        .returning((dsl::bot_msg_channel_id, dsl::bot_msg))
        .get_result(&mut sql)
        .optional()?;

    let (channel_id, bot_msg): (i64, i64) = match res {
        Some(x) => x,
        None => return Ok(false),
    };

    let channel_id = ChannelId(channel_id as u64);
    let bot_msg = MessageId(bot_msg as u64);

    tri!(
        channel_id.delete_message(context, bot_msg).await,
        "Error deleating group watch message"
    );

    Ok(true)
}
