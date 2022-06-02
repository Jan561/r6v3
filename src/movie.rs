use crate::sql::movie::NewMovieChannel;
use crate::sql::uuid::Uuid;
use crate::sql::SqlKey;
use crate::SimpleResult;
use chrono::Utc;
use diesel::prelude::*;
use log::warn;
use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::model::id::UserId;
use serenity::model::mention::Mention;

pub const MOVIE_URIS: [&str; 1] = ["https://www.disneyplus.com/groupwatch/"];
const DEFAULT_MOVIE_TIME_VC: i64 = 0;

pub fn groupwatch_create_msg(uri: impl AsRef<str>, creator: UserId) -> String {
    return format!(
        "GroupWatch: {}\n\nCreated by: {}",
        uri.as_ref(),
        Mention::from(creator),
    );
}

pub async fn handle_groupwatch_default_channel(ctx: &Context, msg: &Message) -> SimpleResult<()> {
    let data = ctx.data.read().await;
    let mut sql = data.get::<SqlKey>().unwrap().connection.get()?;

    use crate::schema::movie_channels::dsl;
    let old_bot_msg: Option<i64> =
        diesel::delete(dsl::movie_channels.filter(dsl::vc.eq(DEFAULT_MOVIE_TIME_VC)))
            .returning(dsl::bot_msg)
            .get_result(&mut sql)
            .optional()?;

    if let Some(old_msg) = old_bot_msg {
        tri!(
            msg.channel_id.delete_message(ctx, old_msg as u64).await,
            "Error deleting old message",
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
        vc: DEFAULT_MOVIE_TIME_VC,
        bot_msg: new_msg.id.0 as i64,
        creator: new_msg.author.id.0 as i64,
        created_at: Utc::now().naive_utc(),
    };

    let result = new_movie_channel.insert(&mut sql)?;

    if !result {
        warn!("Failed storing new group watch for default channel in DB, deleting it.");
        new_msg.delete(ctx).await?;
    }

    Ok(())
}
