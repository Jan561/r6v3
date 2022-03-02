use crate::azure::{resource_group, subscription, vm_name, AzureId, AzureName};
use serenity::prelude::TypeMapKey;

pub struct ConfigKey;

pub struct Config {
    pub subscription: AzureId,
    pub rg: AzureName,
    pub vm: AzureName,
}

impl Config {
    pub fn from_env() -> Config {
        let subscription = subscription();
        let rg = resource_group();
        let vm = vm_name();

        Config {
            subscription,
            rg,
            vm,
        }
    }
}

impl TypeMapKey for ConfigKey {
    type Value = Config;
}
