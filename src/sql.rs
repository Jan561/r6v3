use serenity::prelude::TypeMapKey;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Pool, Sqlite};

use crate::SimpleResult;

pub mod model;

#[derive(Debug, Clone)]
pub struct Sql {
    connection: Pool<Sqlite>,
}

impl Sql {
    pub async fn new() -> SimpleResult<Self> {
        let connection = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(
                SqliteConnectOptions::new()
                    .filename("database.sql")
                    .create_if_missing(true),
            )
            .await?;

        sqlx::migrate!().run(&connection).await?;

        Ok(Self { connection })
    }
}

pub struct SqlKey;

impl TypeMapKey for SqlKey {
    type Value = Sql;
}

pub struct NotInserted;
pub struct Inserted {
    connection: Sql,
}
