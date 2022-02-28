use crate::azure::authentication::TokenScope;
use azure_core::auth::TokenResponse;
use chrono::{Duration, Utc};
use oauth2::AccessToken;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Default, Clone)]
pub struct TokenStore {
    store: Arc<RwLock<HashMap<TokenScope, TokenResponse>>>,
}

impl TokenStore {
    pub async fn token(&self, scope: TokenScope) -> Option<AccessToken> {
        self.token_response(scope)
            .await
            .filter(|r| r.expires_on.signed_duration_since(Utc::now()) > Duration::zero())
            .map(|r| r.token)
    }

    pub async fn token_response(&self, scope: TokenScope) -> Option<TokenResponse> {
        self.store.read().await.get(&scope).cloned()
    }

    pub async fn insert_token(
        &self,
        scope: TokenScope,
        token: TokenResponse,
    ) -> Option<TokenResponse> {
        self.store.write().await.insert(scope, token)
    }
}
