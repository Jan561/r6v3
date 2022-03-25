use crate::schema::ts_members;
use crate::{SimpleError, SimpleResult};
use diesel::prelude::*;
use diesel::{Identifiable, Insertable, Queryable, SqliteConnection};

#[derive(Queryable, Insertable, Identifiable)]
#[diesel(table_name = ts_members)]
#[diesel(primary_key(user_id, removal_pending))]

pub struct TsMember {
    pub user_id: i64,
    pub client_uuid: String,
    pub insertion_pending: bool,
    pub removal_pending: bool,
}

impl TsMember {
    pub fn insert(&self, sql: &mut SqliteConnection) -> SimpleResult<bool> {
        diesel::insert_into(ts_members::table)
            .values(self)
            .on_conflict_do_nothing()
            .execute(sql)
            .map(|rows_affected| rows_affected != 0)
            .map_err(SimpleError::DieselError)
    }

    pub fn schedule_deletion(user_id: i64, sql: &mut SqliteConnection) -> SimpleResult<bool> {
        let mut rows_affected = diesel::delete(
            ts_members::table
                .filter(ts_members::user_id.eq(user_id))
                .filter(ts_members::removal_pending.eq(false))
                .filter(ts_members::insertion_pending.eq(true)),
        )
        .execute(sql)
        .map_err(SimpleError::DieselError)?;

        if rows_affected == 0 {
            rows_affected = diesel::update(
                ts_members::table
                    .filter(ts_members::user_id.eq(user_id))
                    .filter(ts_members::removal_pending.eq(false))
                    .filter(ts_members::insertion_pending.eq(false)),
            )
            .set(ts_members::removal_pending.eq(true))
            .execute(sql)
            .map_err(SimpleError::DieselError)?;
        }

        Ok(rows_affected != 0)
    }
}
