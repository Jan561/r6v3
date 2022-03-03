use crate::azure::management::vm::vm;
use crate::azure::management::{api_version, send_request};
use crate::azure::{AzureClient, AzureName, SubscriptionId};
use crate::SimpleResult;
use async_trait::async_trait;
use http::Request;
use serde::Serialize;

const API_VERSION: &str = "2021-07-01";

macro_rules! run_command {
    ($sub:expr, $rg:expr, $vm:expr) => {
        vm!($sub, $rg, $vm, "runCommand")
    };
}

#[async_trait]
pub trait VmRunCmdClient {
    async fn run(
        &self,
        subscription: &SubscriptionId,
        rg: &AzureName,
        vm: &AzureName,
        cmd: impl Into<Command> + Send + 'static,
    ) -> SimpleResult<()>;
}

#[async_trait]
impl VmRunCmdClient for AzureClient {
    async fn run(
        &self,
        subscription: &SubscriptionId,
        rg: &AzureName,
        vm: &AzureName,
        cmd: impl Into<Command> + Send + 'static,
    ) -> SimpleResult<()> {
        let url: String = run_command!(subscription, rg, vm) + &api_version!(API_VERSION);

        let request = Request::post(url)
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&cmd.into())?.into_bytes().into())
            .expect("Error creating request.");

        send_request(self, request.into()).await.map(|_| ())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Command {
    #[serde(rename = "commandId")]
    pub command_id: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub script: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ShellCommand {
    pub script: Vec<String>,
}

impl From<ShellCommand> for Command {
    fn from(cmd: ShellCommand) -> Command {
        Command {
            command_id: "RunShellScript".to_owned(),
            parameters: vec![],
            script: cmd.script,
        }
    }
}
