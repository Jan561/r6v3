#[macro_use]
extern crate diesel;

#[macro_use]
mod macros;

mod azure;
mod command;
mod conf;
mod handler;
mod hook;
mod movie;
mod owners;
mod permission;
mod schema;
mod sql;
mod voice;

use crate::azure::authentication::{load_cert, load_priv_key};
use crate::azure::{new_azure_client, AzureClientKey};
use crate::command::ping::PING_COMMAND;
use crate::command::start::START_COMMAND;
use crate::command::stop::STOP_COMMAND;
use crate::command::{InstanceLockKey, CMD_PREFIX};
use crate::conf::{ConfigKey, Settings};
use crate::handler::Handler;
use crate::hook::{after_hook, before_hook};
use crate::owners::Owners;
use crate::permission::rbac::{RbacKey, RbacManager};
use crate::sql::{Sql, SqlKey};
use azure_core::HttpError;
use config::ConfigError;
use http::header::ToStrError;
use log::error;
use serenity::client::{Client, ClientBuilder};
use serenity::framework::standard::macros::group;
use serenity::framework::standard::StandardFramework;
use serenity::http::Http;
use serenity::model::id::UserId;
use serenity::model::prelude::CurrentApplicationInfo;
use serenity::prelude::{GatewayIntents, SerenityError, TypeMap};
use std::collections::HashSet;

#[derive(thiserror::Error, Debug)]
pub enum SimpleError {
    #[error("Discord Client Error: {}", .0)]
    SerenityError(#[from] SerenityError),
    #[error("Azure API Error: {}", .0)]
    AzCoreError(#[from] azure_core::Error),
    #[error("OpenSSL Error: {}", .0)]
    OpenSslError(#[from] openssl::error::ErrorStack),
    #[error("IO Error: {}", .0)]
    IoError(#[from] std::io::Error),
    #[error("JWT Error: {}", .0)]
    JwtError(#[from] jwt::Error),
    #[error("Serde Error: {}", .0)]
    SerdeError(#[from] serde_json::Error),
    #[error("Timeout")]
    Timeout,
    #[error("TCP connection not established")]
    NotConnected,
    #[error("Error parsing header: {}", .0)]
    ToStrError(#[from] ToStrError),
    #[error("Config error: {}", .0)]
    ConfigError(#[from] ConfigError),
    #[error("{}", .0)]
    UsageError(String),
    #[error("DB connection error: {}", .0)]
    DbConnectionError(#[from] diesel::result::ConnectionError),
    #[error("Diesel Error: {}", .0)]
    DieselError(#[from] diesel::result::Error),
    #[error("R2D2 Error: {}", .0)]
    R2D2Error(#[from] diesel::r2d2::PoolError),
}

impl From<HttpError> for SimpleError {
    fn from(err: HttpError) -> SimpleError {
        SimpleError::AzCoreError(azure_core::Error::Http(err))
    }
}

pub type SimpleResult<T> = Result<T, SimpleError>;

#[group]
#[commands(ping, start, stop)]
#[only_in(guilds)]
struct General;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().unwrap();
    env_logger::init();

    let config = Settings::new().expect("Error reading config");
    let token = &config.discord_token;

    let http = http(token.as_str());
    let app_info = app_info(&http).await;
    let owners = owners(&app_info);

    let framework = StandardFramework::new()
        .configure(|c| c.prefix(CMD_PREFIX))
        .group(&GENERAL_GROUP)
        .before(before_hook)
        .after(after_hook);

    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

    let mut client = ClientBuilder::new_with_http(http, intents)
        .event_handler(Handler::default())
        .framework(framework)
        .await
        .expect("Error creating client");

    let sql = Sql::new().expect("Failed to initialize Sql.");

    data_w(&client, |data| {
        data.insert::<Owners>(owners);
        data.insert::<AzureClientKey>(new_azure_client(reqwest::Client::new(), &config.azure));
        data.insert::<ConfigKey>(config);
        data.insert::<RbacKey>(RbacManager::new().expect("Error creating rbac manager."));
        data.insert::<InstanceLockKey>(Default::default());
        data.insert::<SqlKey>(sql);
    })
    .await;

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        error!(
            "FATAL: An error occurred while running the client: {:?}",
            why
        );
    }
}

fn http(token: &str) -> Http {
    Http::new(token)
}

async fn app_info(http: &Http) -> CurrentApplicationInfo {
    http.get_current_application_info()
        .await
        .expect("Error getting app info.")
}

fn owners(app_info: &CurrentApplicationInfo) -> HashSet<UserId> {
    let mut set = HashSet::new();
    set.insert(app_info.owner.id);
    set
}

async fn data_w<F: FnOnce(&mut TypeMap)>(client: &Client, f: F) {
    let mut data = client.data.write().await;
    f(&mut data);
}
