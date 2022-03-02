use crate::azure::authentication::TokenScope;
use crate::azure::{AzureClient, AzureName, SubscriptionId};
use crate::SimpleResult;
use async_trait::async_trait;
use azure_core::{Body, Response};
use http::Request;

pub const API_VERSION: &str = "2021-11-01";

macro_rules! api {
    () => {
        "https://management.azure.com"
    };
}

macro_rules! uri {
    ($($part:expr),*) => {
        [$(&*$part),*].join("/") + "?api-version=" + API_VERSION
    }
}

macro_rules! base {
    ($sub:expr, $rg:expr, $($part:expr),*) => {
        uri![api!(), "subscriptions", $sub, "resourceGroups", $rg, $($part),*]
    }
}

macro_rules! compute {
    ($sub:expr, $rg:expr, $($part:expr),*) => {
        base!($sub, $rg, "providers/Microsoft.Compute", $($part),*)
    }
}

macro_rules! vm {
    ($sub:expr, $rg:expr, $vm:expr, $($part:expr),*) => {
        compute!($sub, $rg, "virtualMachines", $vm, $($part),*)
    }
}

#[async_trait]
pub trait VmClient {
    async fn start(
        &self,
        subscription: &SubscriptionId,
        rg: &AzureName,
        name: &AzureName,
    ) -> SimpleResult<()>;
    async fn deallocate(
        &self,
        subscription: &SubscriptionId,
        rg: &AzureName,
        name: &AzureName,
    ) -> SimpleResult<()>;
}

#[async_trait]
impl VmClient for AzureClient {
    async fn start(
        &self,
        subscription: &SubscriptionId,
        rg: &AzureName,
        name: &AzureName,
    ) -> SimpleResult<()> {
        let url = vm!(subscription, rg, name, "start");

        let request = Request::post(url)
            .body(Default::default())
            .expect("Failed building http request.")
            .into();

        send_request(self, request).await.map(|_| ())
    }

    async fn deallocate(
        &self,
        subscription: &SubscriptionId,
        rg: &AzureName,
        name: &AzureName,
    ) -> SimpleResult<()> {
        let url = vm!(subscription, rg, name, "deallocate");

        let request = Request::post(url)
            .body(Default::default())
            .expect("Failed building http request.")
            .into();

        send_request(self, request).await.map(|_| ())
    }
}

async fn send_request(
    client: &AzureClient,
    mut request: azure_core::Request,
) -> SimpleResult<Response> {
    add_content_length_to_header(&mut request);
    client
        .send_authorized_request(request, TokenScope::Management)
        .await
}

fn add_content_length_to_header(request: &mut azure_core::Request) {
    if let Some(len) = content_length(request) {
        request.headers_mut().insert("Content-Length", len.into());
    }
}

fn content_length(request: &azure_core::Request) -> Option<usize> {
    if let Body::Bytes(body) = request.body() {
        Some(body.len())
    } else {
        None
    }
}
