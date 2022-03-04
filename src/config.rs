use crate::azure::{resource_group, subscription, vm_name, AzureId, AzureName};
use serenity::prelude::TypeMapKey;
use std::env;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

const MC_START_SCRIPT_ENV: &str = "R6V3_MC_START_SCRIPT";

pub struct ConfigKey;

pub struct Config {
    pub subscription: AzureId,
    pub rg: AzureName,
    pub vm: AzureName,
    pub mc_start_script: PathBuf,
}

impl Config {
    pub fn from_env() -> Config {
        let subscription = subscription();
        let rg = resource_group();
        let vm = vm_name();
        let mc_start_script = Config::start_script();

        Config {
            subscription,
            rg,
            vm,
            mc_start_script,
        }
    }

    fn start_script() -> PathBuf {
        let file_path: PathBuf = env::var(MC_START_SCRIPT_ENV)
            .expect("MC start script not in env.")
            .into();

        if file_path.is_relative() {
            let mut script = env::current_exe().unwrap();
            script.pop();
            script.push(&file_path);
            script
        } else {
            file_path
        }
    }
}

impl TypeMapKey for ConfigKey {
    type Value = Config;
}

#[derive(Debug, Clone)]
pub struct User {
    user: String,
}

impl User {
    const ALLOWED_CHARS: [char; 2] = ['_', '-'];

    fn valid_name(s: &str) -> bool {
        s.chars()
            .all(|c| c.is_alphanumeric() || User::ALLOWED_CHARS.contains(&c))
    }
}

impl Display for User {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.user.fmt(f)
    }
}

impl From<String> for User {
    fn from(user: String) -> User {
        if !User::valid_name(&user) {
            panic!("Username not valid: {}!", user);
        }

        User {
            user: user.to_lowercase(),
        }
    }
}
