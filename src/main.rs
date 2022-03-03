mod azure;
mod command;
mod config;
mod handler;
mod hook;
mod owners;
mod permission;

use crate::azure::authentication::{load_cert, load_priv_key};
use crate::azure::{new_azure_client, AzureClientKey};
use crate::command::ping::PING_COMMAND;
use crate::command::start::START_COMMAND;
use crate::command::stop::STOP_COMMAND;
use crate::config::{Config, ConfigKey};
use crate::handler::Handler;
use crate::hook::{after_hook, before_hook};
use crate::owners::Owners;
use azure_core::HttpError;
use serenity::client::Client;
use serenity::framework::standard::{macros::group, StandardFramework};
use serenity::http::Http;
use serenity::model::id::UserId;
use serenity::model::prelude::CurrentApplicationInfo;
use serenity::prelude::{SerenityError, TypeMap};
use std::collections::HashSet;
use std::env;
use tokio::sync::RwLockWriteGuard;

const ENV_DISCORD_TOKEN: &str = "DISCORD_TOKEN";

#[derive(thiserror::Error, Debug)]
pub enum SimpleError {
    #[error("Discord Client Error")]
    SerenityError(#[from] SerenityError),
    #[error("Azure API Error")]
    AzCoreError(#[from] azure_core::Error),
    #[error("OpenSSL Error")]
    OpenSslError(#[from] openssl::error::ErrorStack),
    #[error("IO Error")]
    IoError(#[from] std::io::Error),
    #[error("JWT Error")]
    JwtError(#[from] jwt::Error),
    #[error("Serde Error")]
    SerdeError(#[from] serde_json::Error),
    #[error("Timeout")]
    Timeout,
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

    let http = http();
    let app_info = app_info(&http).await;
    let owners = owners(app_info);
    let config = Config::from_env();

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
        .group(&GENERAL_GROUP)
        .before(before_hook)
        .after(after_hook);

    // Login with a bot token from the environment
    let token = discord_token();
    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    data_w(&client, |data| {
        data.insert::<Owners>(owners);
        data.insert::<AzureClientKey>(new_azure_client(reqwest::Client::new()));
        data.insert::<ConfigKey>(config);
    })
    .await;

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

fn http() -> Http {
    Http::new_with_token(&discord_token())
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

fn discord_token() -> String {
    env::var(ENV_DISCORD_TOKEN).expect("Discord Token not in env.")
}

async fn data_w<F: FnOnce(&mut RwLockWriteGuard<TypeMap>)>(client: &Client, f: F) {
    let mut data = client.data.write().await;
    f(&mut data);
}
