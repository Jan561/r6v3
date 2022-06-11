use super::uuid::Uuid;
use crate::schema::movie_channels;
use crate::schema::movie_channels::dsl;
use crate::{SimpleError, SimpleResult};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::{Insertable, Queryable, SqliteConnection};
use serenity::model::id::ChannelId;

#[derive(Insertable, Debug, Clone, Copy)]
#[diesel(table_name = movie_channels)]
pub struct NewMovieChannel<'a> {
    pub id: Uuid,
    pub uri: &'a str,
    pub vc: i64,
    pub bot_msg_channel_id: i64,
    pub bot_msg: i64,
    pub guild: i64,
    pub creator: i64,
    pub created_at: NaiveDateTime,
}

impl NewMovieChannel<'_> {
    pub fn insert(&self, sql: &mut SqliteConnection) -> SimpleResult<Option<Uuid>> {
        diesel::insert_into(dsl::movie_channels)
            .values(self)
            .on_conflict_do_nothing()
            .returning(dsl::id)
            .get_result(sql)
            .optional()
            .map_err(SimpleError::DieselError)
    }
}

#[derive(Queryable, Identifiable, Debug, Clone, Copy)]
#[diesel(table_name = movie_channels)]
pub struct MovieChannelId {
    id: Uuid,
}

impl From<Uuid> for MovieChannelId {
    fn from(id: Uuid) -> Self {
        Self { id }
    }
}

pub fn uuid_from_vc(sql: &mut SqliteConnection, vc: ChannelId) -> SimpleResult<Option<Uuid>> {
    dsl::movie_channels
        .select(dsl::id)
        .filter(dsl::vc.eq(vc.0 as i64))
        .get_result::<Uuid>(sql)
        .optional()
        .map_err(Into::into)
}
