use crate::schema::movie_channels;
use crate::schema::movie_channels::dsl;
use crate::{SimpleError, SimpleResult};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::{Insertable, Queryable, SqliteConnection};

use super::uuid::UUID;

#[derive(Queryable)]
#[diesel(table_name = movie_channels)]
#[diesel(primary_key(id))]
pub struct MovieChannel {
    pub id: UUID,
    pub uri: String,
    pub vc: i64,
    pub bot_msg: i64,
    pub creator: i64,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Debug, Clone, Copy)]
#[diesel(table_name = movie_channels)]
pub struct NewMovieChannel<'a> {
    pub id: UUID,
    pub uri: &'a str,
    pub vc: i64,
    pub bot_msg: i64,
    pub creator: i64,
    pub created_at: NaiveDateTime,
}

impl NewMovieChannel<'_> {
    pub fn insert(&self, sql: &mut SqliteConnection) -> SimpleResult<bool> {
        diesel::insert_into(dsl::movie_channels)
            .values(self)
            .on_conflict_do_nothing()
            .execute(sql)
            .map(|rows_affected| rows_affected != 0)
            .map_err(SimpleError::DieselError)
    }
}

#[derive(Queryable, Identifiable, Debug, Clone, Copy)]
#[diesel(table_name = movie_channels)]
pub struct MovieChannelId {
    id: UUID,
}

impl From<UUID> for MovieChannelId {
    fn from(id: UUID) -> Self {
        Self { id }
    }
}
