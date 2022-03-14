use crate::SimpleResult;
use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::prelude::TypeMapKey;
use std::borrow::Cow;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

pub mod ping;
pub mod start;
pub mod stop;

pub const CMD_PREFIX: &str = "~";

pub struct ProgressMessage<'a> {
    user_msg: &'a Message,
    bot_msg: Option<Message>,
}

impl<'a> ProgressMessage<'a> {
    pub fn new(user_msg: &'a Message) -> ProgressMessage<'a> {
        ProgressMessage {
            user_msg,
            bot_msg: None,
        }
    }

    pub async fn update(&mut self, ctx: &Context, msg: impl ToString) -> SimpleResult<()> {
        let res = match self.bot_msg {
            Some(ref mut existing) => existing.edit(ctx, |m| m.content(msg)).await,
            None => {
                let msg = self.user_msg.reply(ctx, msg.to_string()).await?;
                self.bot_msg = Some(msg);
                Ok(())
            }
        };

        res.map_err(Into::into)
    }
}

pub struct StartStopLockKey;

impl TypeMapKey for StartStopLockKey {
    type Value = StartStopLocks;
}

#[derive(Default)]
pub struct StartStopLocks(RwLock<HashMap<String, StartStopLock>>);

impl StartStopLocks {
    pub async fn get<'a>(&self, key: impl Into<Cow<'a, str>>) -> StartStopLock {
        let key = key.into();
        match self.try_get(&key).await {
            Some(lock) => lock,
            None => self.create(key).await,
        }
    }

    pub async fn try_get(&self, key: impl AsRef<str>) -> Option<StartStopLock> {
        let locks = self.0.read().await;
        locks.get(key.as_ref()).cloned()
    }

    pub async fn create(&self, key: impl ToString) -> StartStopLock {
        let mut locks = self.0.write().await;
        let entry = locks.entry(key.to_string());
        match entry {
            Entry::Occupied(lock) => lock.get().clone(),
            Entry::Vacant(e) => {
                let lock = Arc::new(Mutex::new(()));
                e.insert(lock.clone());
                lock
            }
        }
    }
}

pub type StartStopLock = Arc<Mutex<()>>;

macro_rules! _start_stop_lock {
    ($data:expr, $instance:expr) => {{
        let locks = $data.get::<$crate::command::StartStopLockKey>().unwrap();
        let lock = locks.get($instance).await;
        lock.try_lock_owned().map_err(|_| {
            SimpleError::UsageError(
                "Command execution blocked by another task, try again later.".to_owned(),
            )
        })
    }};
}

use _start_stop_lock as start_stop_lock;

macro_rules! _tri {
    ($res:expr, $log:expr) => {
        if let Err(why) = $res {
            log::warn!("{}: {:?}.", $log, why);
        }
    };
}

use _tri as tri;

macro_rules! _progress {
    ($progress:expr, $ctx:expr, $msg:expr) => {
        $crate::command::tri!(
            $progress.update($ctx, $msg).await,
            "Error updating progress message"
        );
    };
}

use _progress as progress;
