use crate::schema::ts_members;
use crate::{SimpleError, SimpleResult};
use diesel::prelude::*;
use diesel::{Identifiable, Insertable, Queryable, SqliteConnection};

#[derive(Queryable, Insertable, Identifiable)]
#[diesel(table_name = ts_members)]
#[diesel(primary_key(user_id, removal_pending, instance))]
pub struct TsMember {
    pub user_id: i64,
    pub client_uuid: String,
    pub insertion_pending: bool,
    pub removal_pending: bool,
    pub instance: String,
}

impl TsMember {
    pub fn insert(&self, sql: &mut SqliteConnection) -> SimpleResult<bool> {
        use crate::schema::ts_members::dsl::*;

        diesel::insert_into(ts_members)
            .values(self)
            .on_conflict_do_nothing()
            .execute(sql)
            .map(|rows_affected| rows_affected != 0)
            .map_err(SimpleError::DieselError)
    }

    pub fn schedule_deletion(
        user_id_: i64,
        instance_: &str,
        sql: &mut SqliteConnection,
    ) -> SimpleResult<bool> {
        use crate::schema::ts_members::dsl::*;

        let mut rows_affected = diesel::delete(
            ts_members
                .filter(user_id.eq(user_id_))
                .filter(removal_pending.eq(false))
                .filter(insertion_pending.eq(true))
                .filter(instance.eq(instance_)),
        )
        .execute(sql)
        .map_err(SimpleError::DieselError)?;

        if rows_affected == 0 {
            rows_affected = diesel::update(
                ts_members
                    .filter(user_id.eq(user_id))
                    .filter(removal_pending.eq(false))
                    .filter(insertion_pending.eq(false))
                    .filter(instance.eq(instance)),
            )
            .set(removal_pending.eq(true))
            .execute(sql)
            .map_err(SimpleError::DieselError)?;
        }

        Ok(rows_affected != 0)
    }

    pub fn delete_removal_pending(
        sql: &mut SqliteConnection,
        instance_: &str,
    ) -> SimpleResult<Vec<(i64, String)>> {
        use crate::schema::ts_members::dsl::*;

        diesel::delete(
            ts_members
                .filter(removal_pending.eq(true))
                .filter(instance.eq(instance_)),
        )
        .returning((user_id, client_uuid))
        .get_results(sql)
        .map_err(Into::into)
    }

    pub fn unset_insertion_pending(
        sql: &mut SqliteConnection,
        instance_: &str,
    ) -> SimpleResult<Vec<(i64, String)>> {
        use crate::schema::ts_members::dsl::*;

        diesel::update(
            ts_members
                .filter(insertion_pending.eq(true))
                .filter(instance.eq(instance_)),
        )
        .set(insertion_pending.eq(false))
        .returning((user_id, client_uuid))
        .get_results(sql)
        .map_err(Into::into)
    }
}

#[derive(Insertable)]
#[diesel(table_name = ts_members)]
pub struct NewTsMember<'a> {
    pub user_id: i64,
    pub client_uuid: &'a str,
    pub instance: &'a str,
}

impl NewTsMember<'_> {
    pub fn insert(&self, sql: &mut SqliteConnection) -> SimpleResult<bool> {
        use crate::schema::ts_members::dsl::*;

        diesel::insert_into(ts_members)
            .values(self)
            .on_conflict_do_nothing()
            .execute(sql)
            .map(|rows_affected| rows_affected != 0)
            .map_err(SimpleError::DieselError)
    }
}
