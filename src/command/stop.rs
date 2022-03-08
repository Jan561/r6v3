use crate::azure::management::vm::VmClient;
use crate::azure::management::vm_run_cmd::{ShellCommand, VmRunCmdClient};
use crate::command::{progress, ProgressMessage};
use crate::permission::HasPermission;
use crate::{AzureClientKey, ConfigKey, Owners};
use async_trait::async_trait;
use serenity::client::Context;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::channel::Message;
use serenity::model::id::UserId;
use std::fs;

#[command]
async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let config = data.get::<ConfigKey>().unwrap();
    let client = data.get::<AzureClientKey>().unwrap();

    let mut progress_message = ProgressMessage::new(msg);

    progress!(progress_message, ctx, "Stopping game server ...");

    let file = fs::read(&config.mc_stop_script)?;
    let script = ShellCommand {
        script: [std::str::from_utf8(&file).unwrap()],
    };

    client
        .run(&config.subscription, &config.rg, &config.vm, script)
        .await?
        .wait()
        .await?;

    client
        .deallocate(&config.subscription, &config.rg, &config.vm)
        .await?;

    progress!(progress_message, ctx, "Stopped the server.");

    Ok(())
}

pub struct StopPermission;

#[async_trait]
impl HasPermission<StopPermission> for UserId {
    async fn has_permission(&self, ctx: &Context, _: &StopPermission) -> bool {
        ctx.data
            .read()
            .await
            .get::<Owners>()
            .unwrap()
            .contains(self)
    }
}
