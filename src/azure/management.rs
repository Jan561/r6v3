pub mod vm;
pub mod vm_rum_cmd;

use crate::azure::authentication::TokenScope;
use crate::azure::AzureClient;
use crate::SimpleResult;
use azure_core::{Body, Response};

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

macro_rules! api_ {
    () => {
        "https://management.azure.com"
    };
}

use api_ as api;

macro_rules! uri_ {
    ($($part:expr),*) => {
        [$(&*$part),*].join("/")
    }
}

use uri_ as uri;

macro_rules! base_ {
    ($sub:expr, $rg:expr, $($part:expr),*) => {
        $crate::azure::management::uri![
            $crate::azure::management::api!(),
            "subscriptions",
            $sub,
            "resourceGroups",
            $rg,
            $($part),*
        ]
    }
}

use base_ as base;

macro_rules! compute_ {
    ($sub:expr, $rg:expr, $($part:expr),*) => {
        $crate::azure::management::base!(
            $sub,
            $rg,
            "providers/Microsoft.Compute",
            $($part),*
        )
    }
}

use compute_ as compute;

macro_rules! api_version_ {
    ($version:expr) => {
        "?api-version=".to_owned() + $version
    };
}

use api_version_ as api_version;
