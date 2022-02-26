use azure_core::HttpClient;
use serenity::prelude::TypeMapKey;
use std::sync::Arc;

pub struct AzureClientKey;

impl TypeMapKey for AzureClientKey {
    type Value = AzureClient;
}

pub struct AzureClient {
    http_client: Arc<dyn HttpClient>,
}

impl AzureClient {
    pub fn new() -> AzureClient {
        let http_client = azure_core::new_http_client();

        AzureClient { http_client }
    }
}
