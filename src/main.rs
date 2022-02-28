mod azure;
mod command;
mod handler;
mod hook;
mod owners;
mod permission;

use command::ping::PING_COMMAND;
use serenity::async_trait;
use serenity::client::{Client, EventHandler};
use serenity::framework::standard::{
    macros::group,
    StandardFramework,
};
use std::collections::HashSet;

use crate::azure::{AzureClient, AzureClientKey};
use crate::owners::Owners;
use serenity::http::Http;
use serenity::model::id::UserId;
use serenity::model::prelude::CurrentApplicationInfo;
use serenity::prelude::{SerenityError, TypeMap};
use std::env;
use tokio::sync::RwLockWriteGuard;

const ENV_DISCORD_TOKEN: &str = "DISCORD_TOKEN";

pub struct SimpleError {
    pub error_type: ErrorType,
    pub msg: Option<String>,
}

pub type SimpleResult<T> = Result<T, SimpleError>;

pub enum ErrorType {
    SerenityError(SerenityError),
}

impl From<SerenityError> for SimpleError {
    fn from(e: SerenityError) -> Self {
        SimpleError {
            error_type: ErrorType::SerenityError(e),
            msg: None,
        }
    }
}

#[group]
#[commands(ping)]
#[only_in(guilds)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {}

#[tokio::main]
async fn main() {
    let http = http();
    let app_info = app_info(&http).await;
    let owners = owners(app_info);

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let token = discord_token();
    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    data_w(&client, |data| {
        data.insert::<Owners>(owners);
        data.insert::<AzureClientKey>(AzureClient::new());
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
