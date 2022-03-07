use crate::azure::{resource_group, subscription, vm_name, AzureId, AzureName};
use serenity::prelude::TypeMapKey;
use std::borrow::Cow;
use std::env;
use std::path::{Path, PathBuf};

const MC_START_SCRIPT_ENV: &str = "R6V3_MC_START_SCRIPT";
const MC_STOP_SCRIPT_ENV: &str = "R6V3_MC_STOP_SCRIPT";
const MC_RCON_ADDR_ENV: &str = "R6V3_MC_RCON_ADDR";
const MC_RCON_SECRET_ENV: &str = "R6V3_MC_RCON_SECRET";

pub struct ConfigKey;

pub struct Config {
    pub subscription: AzureId,
    pub rg: AzureName,
    pub vm: AzureName,
    pub mc_start_script: PathBuf,
    pub mc_stop_script: PathBuf,
    pub mc_rcon_socket: String,
    pub mc_rcon_secret: String,
}

impl Config {
    pub fn from_env() -> Config {
        let subscription = subscription();
        let rg = resource_group();
        let vm = vm_name();
        let mc_start_script = Config::start_script();
        let mc_stop_script = Config::stop_script();
        let mc_rcon_socket = Config::mc_rcon_socket();
        let mc_rcon_secret = Config::mc_rcon_secret();

        Config {
            subscription,
            rg,
            vm,
            mc_start_script,
            mc_stop_script,
            mc_rcon_socket,
            mc_rcon_secret,
        }
    }

    fn start_script() -> PathBuf {
        let file_path: PathBuf = env::var(MC_START_SCRIPT_ENV)
            .expect("MC start script not in env.")
            .into();

        Config::absolute_path(file_path)
    }

    fn stop_script() -> PathBuf {
        let file_path: PathBuf = env::var(MC_STOP_SCRIPT_ENV)
            .expect("MC stop script not in env.")
            .into();

        Config::absolute_path(file_path)
    }

    fn absolute_path<'a>(p: impl Into<Cow<'a, Path>>) -> PathBuf {
        let p = p.into();

        if p.is_relative() {
            let mut path = env::current_exe().unwrap();
            path.pop();
            path.push(&p);
            path
        } else {
            p.into_owned()
        }
    }

    fn mc_rcon_socket() -> String {
        env::var(MC_RCON_ADDR_ENV).expect("Minecraft rcon address not in env.")
    }

    fn mc_rcon_secret() -> String {
        env::var(MC_RCON_SECRET_ENV).expect("Minecraft rcon password not in env.")
    }
}

impl TypeMapKey for ConfigKey {
    type Value = Config;
}
