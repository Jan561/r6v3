use crate::SimpleResult;
use serenity::client::Context;
use serenity::model::channel::Message;

pub mod ping;
pub mod start;
pub mod stop;

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
