use crate::azure::management::vm::VmClient;
use crate::azure::management::vm_run_cmd::{ShellCommand, VmRunCmdClient};
use crate::command::{progress, ProgressMessage};
use crate::permission::rbac::{HasRbacPermission, RbacPermission};
use crate::permission::HasPermission;
use crate::{AzureClientKey, ConfigKey, RbacKey, SimpleError};
use async_trait::async_trait;
use serenity::client::Context;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::channel::Message;
use serenity::model::id::UserId;
use std::fs;
use std::time::{Duration, SystemTime};
use tokio::time::sleep;

const TIMEOUT: Duration = Duration::from_secs(120);

#[command]
async fn start(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let config = data.get::<ConfigKey>().unwrap();
    let client = data.get::<AzureClientKey>().unwrap();

    let mut progress = ProgressMessage::new(msg);

    progress!(progress, ctx, "Booting the server ...");

    let server_conf = &config.servers["mc"];

    // Booting the server
    client
        .start(
            &server_conf.vm.sub,
            &server_conf.vm.rg,
            &server_conf.vm.name,
        )
        .await?
        .wait()
        .await?;

    progress!(progress, ctx, "Server booted. Waiting for agent ...");

    // Waiting for server to be ready, or timeout after 120 seconds
    let loop_start = SystemTime::now();

    loop {
        if SystemTime::now().duration_since(loop_start).unwrap() > TIMEOUT {
            return Err(SimpleError::Timeout.into());
        }

        let instance = client
            .instance_view(
                &server_conf.vm.sub,
                &server_conf.vm.rg,
                &server_conf.vm.name,
            )
            .await?;
        if instance
            .vm_agent
            .map(|agent| {
                agent
                    .statuses
                    .get(0)
                    .map_or(false, |s| s.display_status == "Ready")
            })
            .unwrap_or(false)
        {
            break;
        }

        sleep(Duration::from_secs(10)).await;
    }

    let file = fs::read(&server_conf.start_script.as_ref().unwrap())?;
    let script = ShellCommand {
        script: [std::str::from_utf8(&file).unwrap()],
    };

    // Fire start command for game server
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

    progress!(progress, ctx, "Started the server.");

    Ok(())
}

pub struct StartPermission;

impl AsRef<str> for StartPermission {
    fn as_ref(&self) -> &str {
        "/mc/start"
    }
}

impl RbacPermission for StartPermission {}

#[async_trait]
impl HasPermission<StartPermission> for UserId {
    async fn has_permission(&self, ctx: &Context, p: &StartPermission) -> bool {
        let data = ctx.data.read().await;
        let rbac = data.get::<RbacKey>().unwrap();
        <Self as HasRbacPermission>::has_permission(self, p, rbac)
    }
}
