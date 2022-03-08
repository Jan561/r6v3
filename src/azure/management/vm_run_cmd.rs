use crate::azure::management::vm::vm;
use crate::azure::management::{api_version, send_request, AsyncTask};
use crate::azure::{AzureClient, AzureName, SubscriptionId};
use crate::SimpleResult;
use async_trait::async_trait;
use http::{Request, StatusCode, Uri};
use serde::ser::SerializeSeq;
use serde::{Serialize, Serializer};
use serde_json::{json, Value};
use std::cell::RefCell;
use std::marker::PhantomData;

const API_VERSION: &str = "2021-07-01";

macro_rules! run_command {
    ($sub:expr, $rg:expr, $vm:expr) => {
        vm!($sub, $rg, $vm, "runCommand")
    };
}

#[async_trait]
pub trait VmRunCmdClient {
    async fn run<C, P, S, U, V>(
        &self,
        subscription: &SubscriptionId,
        rg: &AzureName,
        vm: &AzureName,
        cmd: C,
    ) -> SimpleResult<CommandTask<'_, C>>
    where
        C: Into<BaseCommand<P, S>> + Command + Send,
        P: AsRef<[U]> + Send,
        U: AsRef<str> + Sync,
        S: AsRef<[V]> + Send,
        V: AsRef<str> + Sync;
}

#[async_trait]
impl VmRunCmdClient for AzureClient {
    async fn run<C, P, S, U, V>(
        &self,
        subscription: &SubscriptionId,
        rg: &AzureName,
        vm: &AzureName,
        cmd: C,
    ) -> SimpleResult<CommandTask<'_, C>>
    where
        C: Into<BaseCommand<P, S>> + Command + Send,
        P: AsRef<[U]> + Send,
        U: AsRef<str> + Sync,
        S: AsRef<[V]> + Send,
        V: AsRef<str> + Sync,
    {
        let url: String = run_command!(subscription, rg, vm) + &api_version!(API_VERSION);
        let cmd = cmd.into();

        let parameters = cmd.parameters.as_ref().iter().map(|p| p.as_ref());
        let script = cmd.script.as_ref().iter().map(|s| s.as_ref());

        struct IteratorAdapter<I> {
            iter: RefCell<I>,
            len: usize,
        }

        impl<I, J> Serialize for IteratorAdapter<I>
        where
            I: Iterator<Item = J>,
            J: Serialize,
        {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                let mut s = serializer.serialize_seq(Some(self.len))?;
                while let Some(item) = self.iter.borrow_mut().next() {
                    s.serialize_element(&item)?;
                }
                s.end()
            }
        }

        let body = json!({
            "commandId": cmd.command_id,
            "parameters": IteratorAdapter {
                iter: RefCell::new(parameters),
                len: cmd.parameters.as_ref().len(),
            },
            "script": IteratorAdapter {
                iter: RefCell::new(script),
                len: cmd.script.as_ref().len()
            }
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
    pub async fn wait(self) -> SimpleResult<C::Output> {
        let response = match self.resp_type {
            ResponseType::Status200(ret) => ret,
            ResponseType::Status202(uri) => {
                let task = AsyncTask {
                    client: self.client,
                    uri,
                };

                task.wait().await?.into_body_string().await
            }
        };

        C::parse_response(response)
    }
}

enum ResponseType {
    Status200(String),
    Status202(Uri),
}
