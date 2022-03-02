use crate::azure::management::VmClient;
use crate::permission::HasPermission;
use crate::{AzureClientKey, ConfigKey, Owners};
use async_trait::async_trait;
use serenity::client::Context;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::channel::Message;
use serenity::model::id::UserId;

#[command]
async fn start(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let config = data.get::<ConfigKey>().unwrap();
    data.get::<AzureClientKey>()
        .unwrap()
        .start(&config.subscription, &config.rg, &config.vm)
        .await?;

    msg.reply(ctx, "Started the server.").await?;

    Ok(())
}

pub struct StartPermission;

#[async_trait]
impl HasPermission<StartPermission> for UserId {
    async fn has_permission(&self, ctx: &Context, _: &StartPermission) -> bool {
        ctx.data
            .read()
            .await
            .get::<Owners>()
            .unwrap()
            .contains(self)
    }
}
