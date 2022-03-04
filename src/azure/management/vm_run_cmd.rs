use crate::azure::management::vm::vm;
use crate::azure::management::{api_version, send_request};
use crate::azure::{AzureClient, AzureName, SubscriptionId};
use crate::SimpleResult;
use async_trait::async_trait;
use http::Request;
use serde_json::json;

const API_VERSION: &str = "2021-07-01";

macro_rules! run_command {
    ($sub:expr, $rg:expr, $vm:expr) => {
        vm!($sub, $rg, $vm, "runCommand")
    };
}

#[async_trait]
pub trait VmRunCmdClient {
    async fn run<'a, 'u, 'v, P, S, U, V>(
        &self,
        subscription: &SubscriptionId,
        rg: &AzureName,
        vm: &AzureName,
        cmd: impl Into<Command<P, S>> + Send + 'a,
    ) -> SimpleResult<()>
    where
        P: IntoIterator<Item = &'u U> + Send,
        U: AsRef<str> + 'u + ?Sized,
        S: IntoIterator<Item = &'v V> + Send,
        V: AsRef<str> + 'v + ?Sized,
        'u: 'a,
        'v: 'a;
}

#[async_trait]
impl VmRunCmdClient for AzureClient {
    async fn run<'a, 'u, 'v, P, S, U, V>(
        &self,
        subscription: &SubscriptionId,
        rg: &AzureName,
        vm: &AzureName,
        cmd: impl Into<Command<P, S>> + Send + 'a,
    ) -> SimpleResult<()>
    where
        P: IntoIterator<Item = &'u U> + Send,
        U: AsRef<str> + 'u + ?Sized,
        S: IntoIterator<Item = &'v V> + Send,
        V: AsRef<str> + 'v + ?Sized,
        'u: 'a,
        'v: 'a,
    {
        let url: String = run_command!(subscription, rg, vm) + &api_version!(API_VERSION);
        let cmd = cmd.into();
        let parameters = cmd
            .parameters
            .into_iter()
            .map(|p| p.as_ref())
            .collect::<Vec<&str>>();
        let script = cmd
            .script
            .into_iter()
            .map(|s| s.as_ref())
            .collect::<Vec<&str>>();

        let body = json!({
            "commandId": cmd.command_id,
            "parameters": parameters,
            "script": script
        });

        let request = Request::post(url)
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&body)?.into_bytes().into())
            .expect("Error creating request.");

        send_request(self, request.into()).await.map(|_| ())
    }
}

#[derive(Debug, Clone)]
pub struct Command<P, S> {
    pub command_id: &'static str,
    pub parameters: P,
    pub script: S,
}

#[derive(Debug, Clone)]
pub struct ShellCommand<S> {
    pub script: S,
}

impl<S> From<ShellCommand<S>> for Command<[&'static str; 0], S> {
    fn from(cmd: ShellCommand<S>) -> Command<[&'static str; 0], S> {
        Command {
            command_id: "RunShellScript",
            parameters: [],
            script: cmd.script,
        }
    }
}
