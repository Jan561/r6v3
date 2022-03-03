use crate::azure::management::vm::VmClient;
use crate::azure::management::vm_rum_cmd::{ShellCommand, VmRunCmdClient};
use crate::permission::HasPermission;
use crate::{AzureClientKey, ConfigKey, Owners, SimpleError};
use async_trait::async_trait;
use log::warn;
use serenity::client::Context;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::channel::Message;
use serenity::model::id::UserId;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

const TIMEOUT: Duration = Duration::from_secs(120);
const MC_START_CMD: &str = "cd /home/mc/new_server && sudo -u mc screen -AmdS mcs ./start.sh";

#[command]
async fn start(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let config = data.get::<ConfigKey>().unwrap();
    let client = data.get::<AzureClientKey>().unwrap();

    client
        .start(&config.subscription, &config.rg, &config.vm)
        .await?;

    let mut bot_msg = msg.reply(ctx, "Booting the server...").await;

    if let Err(why) = &bot_msg {
        warn!("Error sending progress message, but continuing.: {}", why);
    }

    let now = SystemTime::now();

    loop {
        if SystemTime::now().duration_since(now).unwrap() > TIMEOUT {
            return Err(SimpleError::Timeout.into());
        }

        let instance = client
            .instance_view(&config.subscription, &config.rg, &config.vm)
            .await?;
        if instance
            .vm_agent
            .map(|agent| agent.statuses.get(0).unwrap().display_status == "Ready")
            .unwrap_or(false)
        {
            break;
        }

        sleep(Duration::from_secs(10));
    }

    if let Ok(msg) = &mut bot_msg {
        if let Err(why) = msg.edit(ctx, |m| m.content("Started the server.")).await {
            warn!("Error updating progress message.: {}", why);
        }
    }

    client
        .run(
            &config.subscription,
            &config.rg,
            &config.vm,
            ShellCommand {
                script: vec![MC_START_CMD.to_owned()],
            },
        )
        .await?;

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
