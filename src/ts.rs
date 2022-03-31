use crate::worker::TsMessage;
use serenity::prelude::TypeMapKey;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::sync::{Mutex, RwLock};

pub struct TsWorkerChannels;

impl TypeMapKey for TsWorkerChannels {
    type Value = Arc<RwLock<HashMap<String, Mutex<Sender<TsMessage>>>>>;
}
