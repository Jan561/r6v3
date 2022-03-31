use crate::conf::TsConfig;
use crate::worker::TsMessage;
use crate::SimpleResult;
use serenity::prelude::TypeMapKey;
use std::collections::HashMap;
use std::net::ToSocketAddrs;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::sync::{Mutex, RwLock};
use ts3_query::managed::{ManagedConfig, ManagedConnection};

const TS_PORT: u16 = 9987;
const TIMEOUT: Duration = Duration::from_secs(60);

pub struct TsWorkerChannels;

impl TypeMapKey for TsWorkerChannels {
    type Value = Arc<RwLock<HashMap<String, Mutex<Sender<TsMessage>>>>>;
}

pub fn connect(conf: &TsConfig) -> SimpleResult<ManagedConnection> {
    let config = ManagedConfig::new(
        conf.address,
        TS_PORT,
        conf.username.clone(),
        conf.password.clone(),
    )?
    .connection_timeout(TIMEOUT);

    ManagedConnection::new(config).map_err(Into::into)
}
