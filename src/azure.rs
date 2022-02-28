mod authentication;

use std::env;
use azure_core::HttpClient;
use serenity::prelude::TypeMapKey;
use std::sync::Arc;

const AZ_DIRECTORY_ENV: &str = "R6V3_AZ_DIRECTORY";
const AZ_TOKEN_ENDPOINT_BASE: &str = "https://login.microsoftonline.com";
const AZ_TOKEN_ENDPOINT_TAIL: &str = "oauth2/v2.0/token";

pub struct AzureClientKey;

impl TypeMapKey for AzureClientKey {
    type Value = AzureClient;
}

pub struct AzureClient {
    http_client: Arc<dyn HttpClient>,
    directory: Directory,
}

impl AzureClient {
    fn token_endpoint(&self) -> String {
        format!("{}/{}/{}", AZ_TOKEN_ENDPOINT_BASE, self.directory.id(), AZ_TOKEN_ENDPOINT_TAIL)
    }
}

pub struct AzureId {
    id: String,
}

impl AzureId {
    pub fn id(&self) -> &str {
        &self.id
    }

    fn valid_string(s: &str) -> bool {
        s.chars().all(AzureId::valid_char)
    }

    fn valid_char(c: char) -> bool {
        c.is_alphanumeric() || c == '-'
    }
}

impl From<String> for AzureId {
    fn from(s: String) -> AzureId {
        if !AzureId::valid_string(&s) {
            panic!("All characters of an azure id must be alphanumeric or '-'.");
        }

        AzureId {
            id: s,
        }
    }
}

pub type Directory = AzureId;
pub type ClientId = AzureId;

impl AzureClient {
    pub fn new(directory: Directory) -> AzureClient {
        let http_client = azure_core::new_http_client();

        AzureClient { http_client, directory }
    }
}

fn azure_directory() -> AzureId {
    AzureId {
        id: env::var(AZ_DIRECTORY_ENV).expect("Azure Directory not found in env."),
    }
}
