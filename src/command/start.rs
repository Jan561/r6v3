use crate::azure::management::vm::VmClient;
use crate::azure::management::vm_run_cmd::{ShellCommand, VmRunCmdClient};
use crate::command::{instance_lock, progress, stop_on_timeout, usage_error, ProgressMessage};
use crate::permission::has_permission;
use crate::permission::rbac::RbacPermission;
use crate::ts::TsWorkerChannels;
use crate::worker::spawn_ts_worker;
use crate::{AzureClientKey, ConfigKey, SimpleError, SimpleResult, CMD_PREFIX};
use log::info;
use serenity::client::Context;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::channel::Message;
use std::fs;
use std::time::{Duration, SystemTime};
use tokio::sync::Mutex;
use tokio::time::sleep;

const TIMEOUT: Duration = Duration::from_secs(120);
const SCRIPT_TIMEOUT: Duration = Duration::from_secs(300);

#[command]
async fn start(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;

    let s_name = server_name(msg)?;

    let _l = instance_lock!(data, s_name)?;

    let config = data.get::<ConfigKey>().unwrap();
    let client = data.get::<AzureClientKey>().unwrap();

    let server_conf = config
        .servers
        .get(s_name)
        .ok_or_else(|| usage_error!("Invalid instance."))?;

    let mut progress = ProgressMessage::new(msg);

    progress!(progress, ctx, "Booting the server ...");
    info!("Booting instance {}.", s_name);

    // Booting the server
    let start_res = client
        .start(
            &server_conf.vm.sub,
            &server_conf.vm.rg,
            &server_conf.vm.name,
        )
        .await?
        .timeout(Some(TIMEOUT))
        .wait()
        .await;

    stop_on_timeout!(
        start_res,
        client,
        &server_conf.vm.sub,
        &server_conf.vm.rg,
        &server_conf.vm.name
    )?;

    progress!(progress, ctx, "Server booted. Waiting for agent ...");
    info!("Successfully booted {}, waiting for agent.", s_name);

    // Waiting for server to be ready, or timeout after 120 seconds
    let loop_start = SystemTime::now();

    let ready = loop {
        if SystemTime::now().duration_since(loop_start).unwrap() > TIMEOUT {
            break Err(SimpleError::Timeout);
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
            break Ok(());
        }

        sleep(Duration::from_secs(10)).await;
    };

    stop_on_timeout!(
        ready,
        client,
        &server_conf.vm.sub,
        &server_conf.vm.rg,
        &server_conf.vm.name
    )?;

    let file = fs::read(&server_conf.start_script.as_ref().unwrap())?;
    let script = ShellCommand {
        script: [std::str::from_utf8(&file).unwrap()],
    };

    progress!(progress, ctx, "Executing start script ...");
    info!("Executing start script on {}.", s_name);

    // Fire start command for game server
    let run_res = client
        .run(
            &server_conf.vm.sub,
            &server_conf.vm.rg,
            &server_conf.vm.name,
            script,
        )
        .await?
        .timeout(Some(SCRIPT_TIMEOUT))
        .wait()
        .await;

    stop_on_timeout!(
        run_res,
        client,
        &server_conf.vm.sub,
        &server_conf.vm.rg,
        &server_conf.vm.name
    )?;

    progress!(progress, ctx, "Started the server.");
    info!("Successfully started {}.", s_name);

    if config.servers[s_name].ts.is_some() {
        let tx = spawn_ts_worker(ctx.clone(), s_name);
        let mut channels = data.get::<TsWorkerChannels>().unwrap().write().await;
        channels.insert(s_name.to_owned(), Mutex::new(tx));
    }

    Ok(())
}

fn server_name(msg: &Message) -> SimpleResult<&str> {
    let offset = CMD_PREFIX.len() + "start".len() + 1;
    if offset < msg.content.len() {
        Ok(&msg.content[offset..])
    } else {
        Err(usage_error!("Syntax: {}start <instance>.", CMD_PREFIX,))
    }
}

pub struct StartPermission(String);

impl StartPermission {
    pub fn from_message(msg: &Message) -> SimpleResult<Self> {
        Ok(StartPermission(server_name(msg)?.to_owned()))
    }
}

impl RbacPermission for StartPermission {
    type T = String;

    fn rbac(&self) -> String {
        format!("/{}/start", self.0)
    }
}

has_permission! { StartPermission }
