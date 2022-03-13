use crate::azure::management::vm::VmClient;
use crate::azure::management::vm_run_cmd::{ShellCommand, VmRunCmdClient};
use crate::command::{progress, ProgressMessage};
use crate::permission::rbac::{HasRbacPermission, RbacPermission};
use crate::permission::HasPermission;
use crate::{AzureClientKey, ConfigKey, RbacKey};
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

    let server_conf = &config.servers["mc"];
    let file = fs::read(&server_conf.stop_script.as_ref().unwrap())?;
    let script = ShellCommand {
        script: [std::str::from_utf8(&file).unwrap()],
    };

    client
        .run(
            &server_conf.vm.sub,
            &server_conf.vm.rg,
            &server_conf.vm.name,
            script,
        )
        .await?
        .wait()
        .await?;

    progress!(progress_message, ctx, "Deallocating server ...");

    client
        .deallocate(
            &server_conf.vm.sub,
            &server_conf.vm.rg,
            &server_conf.vm.name,
        )
        .await?
        .wait()
        .await?;

    progress!(progress_message, ctx, "Stopped the server.");

    Ok(())
}

pub struct StopPermission;

impl AsRef<str> for StopPermission {
    fn as_ref(&self) -> &str {
        "/mc/stop"
    }
}

impl RbacPermission for StopPermission {}

#[async_trait]
impl HasPermission<StopPermission> for UserId {
    async fn has_permission(&self, ctx: &Context, p: &StopPermission) -> bool {
        let data = ctx.data.read().await;
        let rbac = data.get::<RbacKey>().unwrap();
        <Self as HasRbacPermission>::has_permission(self, p, rbac)
    }
}
