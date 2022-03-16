mod azure;
mod command;
mod conf;
mod handler;
mod hook;
mod owners;
mod permission;

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
use azure_core::HttpError;
use config::ConfigError;
use http::header::ToStrError;
use log::error;
use serenity::client::Client;
use serenity::framework::standard::{macros::group, StandardFramework};
use serenity::http::Http;
use serenity::model::id::UserId;
use serenity::model::prelude::CurrentApplicationInfo;
use serenity::prelude::{SerenityError, TypeMap};
use std::collections::HashSet;
use tokio::sync::RwLockWriteGuard;

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
    env_logger::init();

    let config = Settings::new().expect("Error reading config");
    let token = &config.discord_token;

    let http = http(token.as_str());
    let app_info = app_info(&http).await;
    let owners = owners(app_info);

    let framework = StandardFramework::new()
        .configure(|c| c.prefix(CMD_PREFIX)) // set the bot's prefix to "~"
        .group(&GENERAL_GROUP)
        .before(before_hook)
        .after(after_hook);

    // Login with a bot token from the environment
    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    data_w(&client, |data| {
        data.insert::<Owners>(owners);
        data.insert::<AzureClientKey>(new_azure_client(reqwest::Client::new(), &config.azure));
        data.insert::<ConfigKey>(config);
        data.insert::<RbacKey>(RbacManager::new().expect("Error creating rbac manager."));
        data.insert::<InstanceLockKey>(Default::default());
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
    Http::new_with_token(token)
}

async fn app_info(http: &Http) -> CurrentApplicationInfo {
    http.get_current_application_info()
        .await
        .expect("Error getting app info.")
}

fn owners(app_info: CurrentApplicationInfo) -> HashSet<UserId> {
    let mut set = HashSet::new();
    set.insert(app_info.owner.id);
    set
}

async fn data_w<F: FnOnce(&mut RwLockWriteGuard<TypeMap>)>(client: &Client, f: F) {
    let mut data = client.data.write().await;
    f(&mut data);
}
