use crate::azure::management::vm::VmClient;
use crate::azure::management::vm_run_cmd::{ShellCommand, VmRunCmdClient};
use crate::permission::HasPermission;
use crate::{AzureClientKey, ConfigKey, Owners};
use async_trait::async_trait;
use serenity::client::Context;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::channel::Message;
use serenity::model::id::UserId;
use std::fs;
// use crate::minecraft::re_try;

#[command]
async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let config = data.get::<ConfigKey>().unwrap();
    // let mc_rcon = data.get::<MinecraftKey>().unwrap();
    let client = data.get::<AzureClientKey>().unwrap();

    // re_try!(mc_rcon, save_all)?;
    // re_try!(mc_rcon, stop)?;

    // mc_rcon.disconnect().await;

    let mut bot_msg = msg.reply(ctx, "Stopping game server ...").await?;

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

    bot_msg
        .edit(ctx, |m| m.content("Stopped the server."))
        .await?;

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
