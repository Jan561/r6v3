use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Pool, Sqlite};

pub mod model;

#[derive(Debug, Clone)]
pub struct Sql {
    connection: Pool<Sqlite>,
}

impl Sql {
    pub async fn new() -> Self {
        let connection = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(
                SqliteConnectOptions::new()
                    .filename("database.sql")
                    .create_if_missing(true),
            )
            .await
            .expect("Couldn't connect to database.");

        sqlx::migrate!()
            .run(&connection)
            .await
            .expect("Couldn't run database migration");

        Self { connection }
    }
}

pub struct NotInserted;
pub struct Inserted {
    connection: Sql,
}
