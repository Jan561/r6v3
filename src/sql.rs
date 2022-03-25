use crate::SimpleResult;
use diesel::connection::Connection;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;
use diesel_migrations::EmbeddedMigrations;
use diesel_migrations::{embed_migrations, MigrationHarness};
use serenity::prelude::TypeMapKey;
use std::env;
use std::time::Duration;

pub mod model;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!();
const DB_CON_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Clone)]
pub struct Sql {
    pub connection: Pool<ConnectionManager<SqliteConnection>>,
}

impl Sql {
    pub fn new() -> SimpleResult<Self> {
        let db = env::var("DATABASE_URL").expect("DATABASE_URL must be set.");
        let mut con = SqliteConnection::establish(&db).unwrap();
        con.run_pending_migrations(MIGRATIONS)
            .expect("DB migrations failed");
        let manager: ConnectionManager<SqliteConnection> = ConnectionManager::new(&db);
        let r2d2 = diesel::r2d2::Pool::builder()
            .max_size(10)
            .connection_timeout(DB_CON_TIMEOUT)
            .build(manager)
            .expect("Failed to initialize connection pool.");
        Ok(Self { connection: r2d2 })
    }
}

pub struct SqlKey;

impl TypeMapKey for SqlKey {
    type Value = Sql;
}
