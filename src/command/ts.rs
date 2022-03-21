use serenity::framework::standard::macros::command;
use serenity::framework::standard::Args;
use serenity::client::Context;
use serenity::model::channel::Message;
use crate::permission::rbac::RbacPermission;


#[command]
async fn stop(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    
    Ok(())
}

pub struct ConnectPermission;

impl RbacPermission for ConnectPermisison {
    type T = &'static str;

    
}
