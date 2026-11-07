//! The Cloudreve auth adapter: credentials on disk and the live signed-in
//! client.
//!
//! Cloning shares the live account, so every use case holding a handle sees a
//! sign-in at once. The token itself is never handed out: it lives inside the
//! SDK client, and storage reads that client through [`CurrentAccount`].

use std::future::Future;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use cloudreve_sdk_api::ApiError;
use cloudreve_sdk_api::api::SiteApi;
use cloudreve_sdk_api::models::user::Token;
use cloudreve_sdk_api::{Client, ClientConfig};
use tokio::sync::oneshot;

use super::*;
use crate::credential::CredentialStore;
use crate::remote::{RemoteError, USER_AGENT, map_api_error};

/// Deadline of one request this adapter sends on its own.
///
/// The SDK hands `with_timeout` to `reqwest`'s `connect_timeout`, which only
/// bounds the TCP handshake: a server that accepts the connection and then
/// answers nothing would hold the caller, and startup, forever.
pub(crate) const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

/// Wait for `work`, but give up once `limit` of real time has passed.
///
/// The clock is an OS thread, not the executor's timer: startup restores the
/// session on the GUI main thread, and for as long as that thread is blocked the
/// dev CDP bridge keeps a worker inside a GTK call, after which the runtime's
/// timer stops firing. A sleep on the runtime would never end exactly then.
async fn bounded<T>(work: impl Future<Output = T>, limit: Duration) -> Option<T> {
    let (tx, rx) = oneshot::channel();
    tokio::task::spawn_blocking(move || {
        std::thread::sleep(limit);
        let _ = tx.send(());
    });
    tokio::select! {
        out = work => Some(out),
        _ = rx => None,
    }
}

/// Run one SDK request under [`REQUEST_TIMEOUT`].
///
/// A request that runs out of time fails like any other network failure, so a
/// caller that can live without the answer sees the same nothing a dropped
/// connection gives it.
pub(crate) async fn within_deadline<T>(
    request: impl Future<Output = Result<T, ApiError>>,
) -> Result<T, RemoteError> {
    match bounded(request, REQUEST_TIMEOUT).await {
        Some(result) => result.map_err(map_api_error),
        None => Err(RemoteError::Unreachable {
            cause: "请求超时".to_string(),
        }),
    }
}

#[derive(Clone)]
pub struct CloudreveAuth {
    store: Arc<dyn CredentialStore>,
    live: Arc<Mutex<Option<ActiveAccount>>>,
    http: reqwest::Client,
    /// Told once when a request proves the stored session is gone, so the shell
    /// can leave the library. Installed by the composition root.
    lost: Arc<Mutex<Option<Arc<dyn Fn() + Send + Sync>>>>,
}

impl CloudreveAuth {
    pub fn new(store: Arc<dyn CredentialStore>) -> Self {
        Self {
            store,
            live: Arc::new(Mutex::new(None)),
            http: reqwest::Client::new(),
            lost: Arc::new(Mutex::new(None)),
        }
    }

    /// Hand over what should run when the server ends the session.
    pub fn on_session_lost(&self, listener: Arc<dyn Fn() + Send + Sync>) {
        *self.lost.lock().unwrap() = Some(listener);
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
    pub(crate) fn persist(&self, base_url: &str, token: &Token) -> Result<(), RemoteError> {
        let raw = serde_json::to_string(token).map_err(|error| RemoteError::Protocol {
            cause: format!("无法序列化登录令牌: {error}"),
        })?;
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
        within_deadline(client.get_site_config("basic"))
            .await
            .ok()
            .and_then(|cfg| cfg.title)
    }
}

impl CurrentAccount for CloudreveAuth {
    fn current(&self) -> Result<ActiveAccount, RemoteError> {
        self.live
            .lock()
            .ok()
            .and_then(|guard| guard.clone())
            .ok_or(RemoteError::NotSignedIn)
    }

    fn session_lost(&self) {
        *self.live.lock().unwrap() = None;
        let listener = self.lost.lock().unwrap().clone();
        if let Some(listener) = listener {
            listener();
        }
    }
}
