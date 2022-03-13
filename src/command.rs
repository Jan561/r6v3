use crate::SimpleResult;
use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::prelude::TypeMapKey;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

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
    type Value = Mutex<HashMap<String, StartStopLock>>;
}

pub type StartStopLock = Arc<Mutex<()>>;

macro_rules! _start_stop_lock {
    ($data:expr, $instance:expr) => {{
        let mut locks = $data
            .get::<$crate::command::StartStopLockKey>()
            .unwrap()
            .lock()
            .await;
        let lock = match locks.get($instance).cloned() {
            Some(l) => l,
            None => {
                let l = Arc::new(Mutex::new(()));
                locks.insert($instance.to_owned(), l.clone());
                l
            }
        };
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
