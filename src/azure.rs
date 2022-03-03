pub mod authentication;
#[macro_use]
pub mod management;

use crate::azure::authentication::token_store::TokenStore;
use crate::azure::authentication::{TokenManager, TokenScope};
use crate::{load_cert, load_priv_key, SimpleError, SimpleResult};
use azure_core::{HttpClient, HttpError, Request, Response};
use log::info;
use oauth2::AccessToken;
use openssl::pkey::{PKey, Private};
use openssl::x509::X509;
use reqwest::Client;
use serenity::prelude::TypeMapKey;
use std::env;
use std::fmt::{Display, Formatter};
use std::ops::Deref;

const AZ_DIRECTORY_ENV: &str = "R6V3_AZ_DIRECTORY";
const AZ_CLIENT_ENV: &str = "R6V3_AZ_CLIENT";
const AZ_SUBSCRIPTION_ENV: &str = "R6V3_AZ_SUBSCRIPTION";
const AZ_RG_ENV: &str = "R6V3_AZ_RG";
const AZ_VM_ENV: &str = "R6V3_AZ_VM";

pub struct AzureClientKey;

impl TypeMapKey for AzureClientKey {
    type Value = AzureClient;
}

pub struct AzureClient {
    token_manager: TokenManager,
    token_store: TokenStore,
    http: Client,
}

impl AzureClient {
    pub fn new(
        directory: Directory,
        client: ClientId,
        cert: X509,
        key: PKey<Private>,
        http: Client,
    ) -> AzureClient {
        AzureClient {
            token_manager: TokenManager::new(directory, client, cert, key),
            token_store: TokenStore::default(),
            http,
        }
    }

    async fn token(&self, scope: TokenScope) -> SimpleResult<AccessToken> {
        if let Some(token) = self.token_store.token(scope).await {
            Ok(token)
        } else {
            info!(
                "Requesting new Azure API Token for scope {}.",
                scope.scope()
            );

            let tr = self.token_manager.request_new(&self.http, scope).await?;
            let token = tr.token.clone();
            self.token_store.insert_token(scope, tr).await;
            Ok(token)
        }
    }

    async fn send_authorized_request(
        &self,
        mut request: Request,
        scope: TokenScope,
    ) -> SimpleResult<Response> {
        self.add_token_to_header(&mut request, scope).await?;
        let response = self.http.execute_request2(&request).await?;
        if response.status().is_success() {
            Ok(response)
        } else {
            Err(SimpleError::AzCoreError(azure_core::Error::Http(
                HttpError::StatusCode {
                    status: response.status(),
                    body: response.into_body_string().await,
                },
            )))
        }
    }

    async fn add_token_to_header(
        &self,
        request: &mut Request,
        scope: TokenScope,
    ) -> SimpleResult<()> {
        let auth_header = "Bearer ".to_string() + self.token(scope).await?.secret();
        request
            .headers_mut()
            .insert("Authorization", auth_header.try_into().unwrap());

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct AzureId {
    id: String,
}

impl AzureId {
    pub fn id(&self) -> &str {
        &self.id
    }

    fn valid_string(s: &str) -> bool {
        s.chars().all(|c| c.is_alphanumeric() || c == '-')
    }
}

impl Display for AzureId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.id().fmt(f)
    }
}

impl Deref for AzureId {
    type Target = str;

    fn deref(&self) -> &str {
        self.id()
    }
}

impl From<String> for AzureId {
    fn from(s: String) -> AzureId {
        if !AzureId::valid_string(&s) {
            panic!("All characters of an azure id must be alphanumeric or '-'.");
        }

        AzureId { id: s }
    }
}

pub type Directory = AzureId;
pub type ClientId = AzureId;
pub type SubscriptionId = AzureId;

#[derive(Debug, Clone)]
pub struct AzureName {
    name: String,
}

impl AzureName {
    const ALLOWED_CHARS: [char; 5] = ['_', '-', '.', '(', ')'];

    pub fn name(&self) -> &str {
        &self.name
    }

    fn valid_string(s: &str) -> bool {
        s.chars()
            .all(|c| c.is_alphanumeric() || AzureName::ALLOWED_CHARS.contains(&c))
    }
}

impl Deref for AzureName {
    type Target = str;

    fn deref(&self) -> &str {
        self.name()
    }
}

impl Display for AzureName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.name().fmt(f)
    }
}

impl From<String> for AzureName {
    fn from(s: String) -> Self {
        if !AzureName::valid_string(&s) {
            panic!(
                "All characters of an azure name must be alphanumeric or '_', '-','.','(', ')'."
            );
        }

        AzureName { name: s }
    }
}

fn directory_id() -> Directory {
    Directory::from(env::var(AZ_DIRECTORY_ENV).expect("Azure Directory not found in env."))
}

fn client_id() -> ClientId {
    ClientId::from(env::var(AZ_CLIENT_ENV).expect("Azure Client not found in env."))
}

pub fn new_azure_client(http: Client) -> AzureClient {
    let x509 = load_cert().expect("Certificate not found.");
    let secret = load_priv_key().expect("Private Key not found.");
    AzureClient::new(directory_id(), client_id(), x509, secret, http)
}

pub fn subscription() -> AzureId {
    env::var(AZ_SUBSCRIPTION_ENV)
        .expect("Subscription id not in env.")
        .into()
}

pub fn resource_group() -> AzureName {
    env::var(AZ_RG_ENV)
        .expect("Resource Group not in env.")
        .into()
}

pub fn vm_name() -> AzureName {
    env::var(AZ_VM_ENV).expect("VM Name not in env.").into()
}
