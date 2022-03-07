use crate::azure::management::vm::vm;
use crate::azure::management::{api_version, send_request};
use crate::azure::{AzureClient, AzureName, SubscriptionId};
use crate::SimpleResult;
use async_trait::async_trait;
use http::{Request, StatusCode, Uri};
use serde_json::{json, Value};
use std::marker::PhantomData;
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
    async fn run<'a, 'u, 'v, C, P, S, U, V>(
        &self,
        subscription: &SubscriptionId,
        rg: &AzureName,
        vm: &AzureName,
        cmd: C,
    ) -> SimpleResult<CommandTask<'_, C>>
    where
        C: Into<BaseCommand<P, S>> + Command + Send + 'a,
        P: IntoIterator<Item = &'u U> + Send,
        U: AsRef<str> + 'u + ?Sized,
        S: IntoIterator<Item = &'v V> + Send,
        V: AsRef<str> + 'v + ?Sized,
        'u: 'a,
        'v: 'a;
}

#[async_trait]
impl VmRunCmdClient for AzureClient {
    async fn run<'a, 'u, 'v, C, P, S, U, V>(
        &self,
        subscription: &SubscriptionId,
        rg: &AzureName,
        vm: &AzureName,
        cmd: C,
    ) -> SimpleResult<CommandTask<'_, C>>
    where
        C: Into<BaseCommand<P, S>> + Command + Send + 'a,
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
                        _c: PhantomData,
                    })
                } else if r.status() == StatusCode::ACCEPTED {
                    let location = r.headers()["location"].to_str()?;
                    Ok(CommandTask {
                        resp_type: ResponseType::Status202(
                            location.try_into().expect("Not a valid uri."),
                        ),
                        client: self,
                        _c: PhantomData,
                    })
                } else {
                    unimplemented!("Received unexpected status code: {}.", r.status())
                }
            }
            Err(e) => Err(e),
        }
    }
}

pub trait Command {
    type Output;

    fn parse_response(body: impl AsRef<str>) -> SimpleResult<Self::Output>;
}

#[derive(Debug, Clone)]
pub struct BaseCommand<P, S> {
    pub command_id: &'static str,
    pub parameters: P,
    pub script: S,
}

#[derive(Debug, Clone)]
pub struct ShellCommand<S> {
    pub script: S,
}

impl<S> From<ShellCommand<S>> for BaseCommand<[&'static str; 0], S> {
    fn from(cmd: ShellCommand<S>) -> BaseCommand<[&'static str; 0], S> {
        BaseCommand {
            command_id: "RunShellScript",
            parameters: [],
            script: cmd.script,
        }
    }
}

impl<S> Command for ShellCommand<S> {
    type Output = String;

    fn parse_response(body: impl AsRef<str>) -> SimpleResult<String> {
        let value: Value = serde_json::from_str(body.as_ref())?;
        let msg = value["value"][0]["message"]
            .as_str()
            .expect("Couldn't parse response.");
        Ok(msg.to_owned())
    }
}

pub struct CommandTask<'a, C> {
    resp_type: ResponseType,
    client: &'a AzureClient,
    _c: PhantomData<C>,
}

impl<'a, C> CommandTask<'a, C>
where
    C: Command,
{
    const POLL_INTERVAL: Duration = Duration::from_secs(3);

    pub async fn wait(self) -> SimpleResult<C::Output> {
        let response = match self.resp_type {
            ResponseType::Status200(ret) => ret,
            ResponseType::Status202(uri) => loop {
                let request = Request::get(&uri)
                    .body(Default::default())
                    .expect("Error creating request.")
                    .into();

                let response = send_request(self.client, request).await?;

                if response.status() == StatusCode::OK {
                    break response.into_body_string().await;
                }

                if response.status() != StatusCode::ACCEPTED {
                    unimplemented!("Received unexpected status code: {}.", response.status());
                }

                sleep(Self::POLL_INTERVAL).await;
            },
        };

        C::parse_response(response)
    }
}

enum ResponseType {
    Status200(String),
    Status202(Uri),
}
