use crate::azure::{AzureId, AzureName, ClientId, Directory};
use crate::SimpleResult;
use bimap::BiMap;
use config::{Config, File, FileFormat};
use lazy_static::lazy_static;
use serde::{Deserialize, Deserializer};
use serenity::model::id::GuildId;
use serenity::prelude::TypeMapKey;
use std::borrow::Cow;
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};

pub type Servers = HashMap<String, ServerConfig>;

fn prepare_path<'a>(path: impl Into<Cow<'a, Path>>) -> PathBuf {
    let path = path.into();

    if path.is_relative() {
        lazy_static! {
            static ref EXE_DIR: PathBuf = {
                let mut p = env::current_exe().unwrap();
                p.pop();
                p
            };
        }

        let mut dir = EXE_DIR.clone();
        dir.push(&path);
        dir
    } else {
        path.into_owned()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct VmConfig {
    pub name: AzureName,
    pub rg: AzureName,
    pub sub: AzureId,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub vm: VmConfig,
    #[serde(deserialize_with = "deserialize_path_opt")]
    pub start_script: Option<PathBuf>,
    #[serde(deserialize_with = "deserialize_path_opt")]
    pub stop_script: Option<PathBuf>,
}

fn deserialize_path_opt<'de, D>(d: D) -> Result<Option<PathBuf>, D::Error>
where
    D: Deserializer<'de>,
{
    let path: Option<PathBuf> = Deserialize::deserialize(d)?;
    Ok(path.map(prepare_path))
}

fn deserialize_path<'de, D>(d: D) -> Result<PathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    PathBuf::deserialize(d)
        .map(prepare_path)
        .map_err(Into::into)
}

#[derive(Debug, Clone, Deserialize)]
pub struct AzureClientConfig {
    pub directory: Directory,
    pub client: ClientId,
    #[serde(deserialize_with = "deserialize_path")]
    pub cert_path: PathBuf,
    #[serde(deserialize_with = "deserialize_path")]
    pub cert_key: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MovieTimeConf {
    pub text_channel: u64,
    pub voice_channel: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub discord_token: String,
    pub azure: AzureClientConfig,
    pub servers: Servers,
    pub guilds: BiMap<String, GuildId>,
    pub movie_time: HashMap<String, MovieTimeConf>,
}

impl Settings {
    pub fn new() -> SimpleResult<Self> {
        let s = Config::builder().add_source(File::new("config.toml", FileFormat::Toml));

        s.build()?.try_deserialize().map_err(Into::into)
    }
}

pub struct ConfigKey;

impl TypeMapKey for ConfigKey {
    type Value = Settings;
}
