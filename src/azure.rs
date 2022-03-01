pub mod authentication;

use crate::azure::authentication::token_store::TokenStore;
use crate::azure::authentication::{TokenManager, TokenScope};
use crate::{load_cert, load_priv_key, SimpleResult};
use oauth2::AccessToken;
use openssl::pkey::{PKey, Private};
use openssl::x509::X509;
use reqwest::Client;
use serenity::prelude::TypeMapKey;
use std::env;

const AZ_DIRECTORY_ENV: &str = "R6V3_AZ_DIRECTORY";
const AZ_CLIENT_ENV: &str = "R6V3_AZ_CLIENT";

pub struct AzureClientKey;

impl TypeMapKey for AzureClientKey {
    type Value = AzureClient;
}

pub struct AzureClient {
    directory: Directory,
    client: ClientId,
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
            token_manager: TokenManager::new(directory.clone(), client.clone(), cert, key),
            token_store: TokenStore::default(),
            directory,
            client,
            http,
        }
    }

    async fn token(&self, scope: TokenScope) -> SimpleResult<AccessToken> {
        if let Some(token) = self.token_store.token(scope).await {
            Ok(token)
        } else {
            let tr = self.token_manager.request_new(&self.http, scope).await?;
            let token = tr.token.clone();
            self.token_store.insert_token(scope, tr).await;
            Ok(token)
        }
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
