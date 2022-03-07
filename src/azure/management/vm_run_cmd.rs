use crate::azure::management::vm::vm;
use crate::azure::management::{api_version, send_request};
use crate::azure::{AzureClient, AzureName, SubscriptionId};
use crate::SimpleResult;
use async_trait::async_trait;
use http::{Request, StatusCode, Uri};
use serde_json::{json, Value};
use std::time::Duration;
use tokio::time::sleep;

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
    ) -> SimpleResult<CommandTask<'_>>
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
    ) -> SimpleResult<CommandTask<'_>>
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
            .body(body.to_string().into_bytes().into())
            .expect("Error creating request.");

        match send_request(self, request.into()).await {
            Ok(r) => {
                if r.status() == StatusCode::OK {
                    Ok(CommandTask {
                        resp_type: ResponseType::Status200(r.into_body_string().await),
                        client: self,
                    })
                } else if r.status() == StatusCode::ACCEPTED {
                    let location = r.headers()["location"].to_str()?;
                    Ok(CommandTask {
                        resp_type: ResponseType::Status202(
                            location.try_into().expect("Not a valid uri."),
                        ),
                        client: self,
                    })
                } else {
                    unimplemented!()
                }
            }
            Err(e) => Err(e),
        }
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

pub struct CommandTask<'a> {
    resp_type: ResponseType,
    client: &'a AzureClient,
}

impl<'a> CommandTask<'a> {
    const POLL_INTERVAL: Duration = Duration::from_secs(3);

    pub async fn wait(self) -> SimpleResult<String> {
        match self.resp_type {
            ResponseType::Status200(ret) => Ok(ret),
            ResponseType::Status202(uri) => loop {
                let request = Request::get(&uri)
                    .body(Default::default())
                    .expect("Error creating request.")
                    .into();

                let response = send_request(self.client, request).await?;
                let body = response.into_body_string().await;

                if !body.is_empty() {
                    let value: Value = serde_json::from_str(&body)?;
                    let msg = value["value"][0]["message"]
                        .as_str()
                        .expect("Error converting message to string.");
                    return Ok(msg.to_owned());
                }

                sleep(CommandTask::POLL_INTERVAL).await;
            },
        }
    }
}

enum ResponseType {
    Status200(String),
    Status202(Uri),
}
