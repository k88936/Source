//! The Cloudreve auth adapter: credentials on disk and the live signed-in
//! client.
//!
//! Cloning shares the live account, so every use case holding a handle sees a
//! sign-in at once. The token itself is never handed out: it lives inside the
//! SDK client, and storage reads that client through [`CurrentAccount`].

use std::sync::{Arc, Mutex};

use cloudreve_sdk_api::api::SiteApi;
use cloudreve_sdk_api::models::user::Token;
use cloudreve_sdk_api::{Client, ClientConfig};

use super::*;
use crate::credential::CredentialStore;
use crate::remote::USER_AGENT;

#[derive(Clone)]
pub struct CloudreveAuth {
    store: Arc<dyn CredentialStore>,
    live: Arc<Mutex<Option<ActiveAccount>>>,
    http: reqwest::Client,
}

impl CloudreveAuth {
    pub fn new(store: Arc<dyn CredentialStore>) -> Self {
        Self {
            store,
            live: Arc::new(Mutex::new(None)),
            http: reqwest::Client::new(),
        }
    }

    pub(crate) fn client_for(&self, base_url: &str) -> Arc<Client> {
        Arc::new(Client::new(
            ClientConfig::new(base_url)
                .with_timeout(60)
                .with_user_agent(USER_AGENT),
        ))
    }

    pub(crate) fn http(&self) -> &reqwest::Client {
        &self.http
    }

    pub(crate) fn live(&self) -> &Mutex<Option<ActiveAccount>> {
        &self.live
    }

    /// The token is the only thing auth adds to what the account row already
    /// says, so it is written under its server and nowhere else.
    pub(crate) fn persist(&self, base_url: &str, token: &Token) -> Result<(), AppError> {
        let raw = serde_json::to_string(token)
            .map_err(|error| AppError::internal(format!("无法序列化登录令牌: {error}")))?;
        self.store.write(base_url, &raw);
        Ok(())
    }

    pub(crate) fn read_token(&self, base_url: &str) -> Option<String> {
        self.store.read(base_url)
    }

    pub(crate) fn clear_token(&self, base_url: &str) {
        self.store.clear(base_url);
    }

    pub(crate) fn install(&self, base_url: &str, client: Arc<Client>) {
        *self.live.lock().unwrap() = Some(ActiveAccount {
            base_url: base_url.to_string(),
            client,
        });
    }

    pub(crate) async fn site_name_of(&self, client: &Client) -> Option<String> {
        match client.get_site_config("basic").await {
            Ok(cfg) => cfg.title,
            Err(_) => None,
        }
    }
}

impl CurrentAccount for CloudreveAuth {
    fn current(&self) -> Result<ActiveAccount, AppError> {
        self.live
            .lock()
            .ok()
            .and_then(|guard| guard.clone())
            .ok_or_else(|| AppError::unauthorized("尚未登录"))
    }
}
