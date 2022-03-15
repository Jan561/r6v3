use crate::azure::management::{api_version, send_request, AsyncTask};
use crate::azure::{AzureClient, AzureName, SubscriptionId};
use crate::SimpleResult;
use async_trait::async_trait;
use http::Request;
use serde::Deserialize;
use std::time::Duration;

const API_VERSION: &str = "2021-11-01";

macro_rules! vm_ {
    ($sub:expr, $rg:expr, $vm:expr, $($part:expr),*) => {
        $crate::azure::management::compute!(
            $sub,
            $rg,
            "virtualMachines",
            $vm,
            $($part),*
        )
    }
}

pub(super) use vm_ as vm;

#[async_trait]
pub trait VmClient {
    async fn start(
        &self,
        subscription: &SubscriptionId,
        rg: &AzureName,
        name: &AzureName,
    ) -> SimpleResult<ActionTask<'_>>;
    async fn deallocate(
        &self,
        subscription: &SubscriptionId,
        rg: &AzureName,
        name: &AzureName,
    ) -> SimpleResult<ActionTask<'_>>;
    async fn instance_view(
        &self,
        subscription: &SubscriptionId,
        rg: &AzureName,
        name: &AzureName,
    ) -> SimpleResult<InstanceView>;
}

#[async_trait]
impl VmClient for AzureClient {
    async fn start(
        &self,
        subscription: &SubscriptionId,
        rg: &AzureName,
        name: &AzureName,
    ) -> SimpleResult<ActionTask<'_>> {
        let url = vm!(subscription, rg, name, "start") + &api_version!(API_VERSION);

        let request = Request::post(url)
            .body(Default::default())
            .expect("Failed building http request.")
            .into();

        let response = send_request(self, request).await?;

        let task = ActionTask {
            task: AsyncTask::new(
                self,
                response.headers()["location"]
                    .to_str()?
                    .try_into()
                    .expect("Not a valid uri."),
            ),
        };

        Ok(task)
    }

    async fn deallocate(
        &self,
        subscription: &SubscriptionId,
        rg: &AzureName,
        name: &AzureName,
    ) -> SimpleResult<ActionTask<'_>> {
        let url = vm!(subscription, rg, name, "deallocate") + &api_version!(API_VERSION);

        let request = Request::post(url)
            .body(Default::default())
            .expect("Failed building http request.")
            .into();

        let response = send_request(self, request).await?;

        let task = ActionTask {
            task: AsyncTask::new(
                self,
                response.headers()["location"]
                    .to_str()?
                    .try_into()
                    .expect("Not a valid uri."),
            ),
        };

        Ok(task)
    }

    async fn instance_view(
        &self,
        subscription: &SubscriptionId,
        rg: &AzureName,
        name: &AzureName,
    ) -> SimpleResult<InstanceView> {
        let url = vm!(subscription, rg, name, "instanceView") + &api_version!(API_VERSION);

        let request = Request::get(url)
            .body(Default::default())
            .expect("Error creating request.")
            .into();

        let response = send_request(self, request).await?;
        let body = response.into_body_string().await;
        serde_json::from_str(&body).map_err(Into::into)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct InstanceView {
    #[serde(rename = "vmAgent", default)]
    pub vm_agent: Option<VmAgent>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VmAgent {
    pub statuses: Vec<Status>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Status {
    #[serde(rename = "displayStatus")]
    pub display_status: String,
}

pub struct ActionTask<'a> {
    task: AsyncTask<'a>,
}

impl<'a> ActionTask<'a> {
    pub fn timeout(self, timeout: Option<Duration>) -> Self {
        Self {
            task: self.task.timeout(timeout),
        }
    }

    pub async fn wait(self) -> SimpleResult<()> {
        self.task.wait().await.map(|_| ())
    }
}
