use crate::azure::management::vm::VmClient;
use crate::azure::management::vm_run_cmd::{ShellCommand, VmRunCmdClient};
use crate::command::{instance_lock, progress, usage_error, ProgressMessage};
use crate::permission::has_permission;
use crate::permission::rbac::RbacPermission;
use crate::ts::TsWorkerChannels;
use crate::worker::TsMessage;
use crate::{AzureClientKey, ConfigKey, SimpleError, SimpleResult, CMD_PREFIX};
use log::{info, warn};
use serenity::client::Context;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::channel::Message;
use std::fs;
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_secs(300);

#[command]
async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;

    let s_name = server_name(msg)?;

    let _l = instance_lock!(data, s_name)?;

    let config = data.get::<ConfigKey>().unwrap();
    let client = data.get::<AzureClientKey>().unwrap();

    let mut progress_message = ProgressMessage::new(msg);

    let server_conf = config
        .servers
        .get(s_name)
        .ok_or_else(|| usage_error!("Invalid instance."))?;

    if server_conf.ts.is_some() {
        let mut channels = data.get::<TsWorkerChannels>().unwrap().write().await;
        match channels.remove(s_name) {
            Some(c) => c
                .lock()
                .await
                .send(TsMessage::Stop)
                .await
                .unwrap_or_else(|_| warn!("TS Worker channel is broken.")),
            None => warn!("TS Worker channel not present, can't stop the worker."),
        }
    }

    progress!(progress_message, ctx, "Executing stop script  ...");
    info!("Executing stop script on {}.", s_name);

    let file = fs::read(&server_conf.stop_script.as_ref().unwrap())?;
    let script = ShellCommand {
        script: [std::str::from_utf8(&file).unwrap()],
    };

    let mut force = false;

    client
        .run(
            &server_conf.vm.sub,
            &server_conf.vm.rg,
            &server_conf.vm.name,
            script,
        )
        .await?
        .timeout(Some(TIMEOUT))
        .wait()
        .await
        .map(|_| ())
        .or_else(|e| match e {
            SimpleError::Timeout => {
                force = true;
                warn!("Failed to shutdown server gracefully: {:?}", e);
                Ok(())
            }
            other => Err(other),
        })?;

    progress!(progress_message, ctx, "Deallocating server ...");
    info!("Deallocating instance {}.", s_name);

    client
        .deallocate(
            &server_conf.vm.sub,
            &server_conf.vm.rg,
            &server_conf.vm.name,
        )
        .await?
        .wait()
        .await?;

    if !force {
        progress!(progress_message, ctx, "Stopped the server.");
    } else {
        progress!(progress_message, ctx, "Stopped the server forcefully.");
    }

    info!("Successfully stopped instance {}.", s_name);

    Ok(())
}

fn server_name(msg: &Message) -> SimpleResult<&str> {
    let offset = CMD_PREFIX.len() + "stop".len() + 1;
    if offset < msg.content.len() {
        Ok(&msg.content[offset..])
    } else {
        Err(usage_error!("Syntax: {}stop <instance>.", CMD_PREFIX,))
    }
}

pub struct StopPermission(String);

impl StopPermission {
    pub fn from_message(msg: &Message) -> SimpleResult<Self> {
        Ok(StopPermission(server_name(msg)?.to_owned()))
    }
}

impl RbacPermission for StopPermission {
    type T = String;

    fn rbac(&self) -> String {
        format!("/{}/stop", self.0)
    }
}

has_permission! { StopPermission }
