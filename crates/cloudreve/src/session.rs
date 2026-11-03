//! Service state: the signed-in session and the service handle.

use super::*;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct PersistedSession {
    pub(crate) base_url: String,
    pub(crate) token: Token,
    pub(crate) user: UserInfo,
    pub(crate) site_name: Option<String>,
}

pub(crate) struct ActiveSession {
    pub(crate) base_url: String,
    pub(crate) client: Arc<Client>,
}

pub struct CloudreveService {
    pub(crate) store: Arc<dyn CredentialStore>,
    pub(crate) active: Mutex<Option<ActiveSession>>,
    pub(crate) http: reqwest::Client,
    pub(crate) sessions: Option<Arc<dyn UploadSessionPort>>,
}

impl CloudreveService {
    pub fn new(store: Arc<dyn CredentialStore>) -> Self {
        Self {
            store,
            active: Mutex::new(None),
            http: reqwest::Client::new(),
            sessions: None,
        }
    }

    /// Persist upload-session/chunk snapshots so interrupted uploads resume
    /// after an app restart (including `content://` sources).
    pub fn with_session_store(mut self, sessions: Arc<dyn UploadSessionPort>) -> Self {
        self.sessions = Some(sessions);
        self
    }

    pub(crate) fn client_for(&self, base_url: &str) -> Arc<Client> {
        Arc::new(Client::new(
            ClientConfig::new(base_url)
                .with_timeout(60)
                .with_user_agent(USER_AGENT),
        ))
    }

    pub(crate) fn current_client(&self) -> Result<Arc<Client>, AppError> {
        self.active
            .lock()
            .ok()
            .and_then(|guard| guard.as_ref().map(|s| s.client.clone()))
            .ok_or_else(|| AppError::unauthorized("尚未登录"))
    }

    pub(crate) fn current_base(&self) -> Result<String, AppError> {
        self.active
            .lock()
            .ok()
            .and_then(|guard| guard.as_ref().map(|s| s.base_url.clone()))
            .ok_or_else(|| AppError::unauthorized("尚未登录"))
    }

    pub(crate) fn install(
        &self,
        base_url: &str,
        client: Arc<Client>,
        user: UserInfo,
        site_name: Option<String>,
        token: Option<Token>,
    ) -> Result<SessionState, AppError> {
        if let Some(token) = &token {
            let persisted = PersistedSession {
                base_url: base_url.to_string(),
                token: token.clone(),
                user: user.clone(),
                site_name: site_name.clone(),
            };
            if let Ok(raw) = serde_json::to_string(&persisted) {
                self.store.write(&raw);
            }
        }
        *self.active.lock().unwrap() = Some(ActiveSession {
            base_url: base_url.to_string(),
            client,
        });
        Ok(SessionState {
            server: base_url.to_string(),
            site_name,
            user,
            library_root: None,
            capacity: None,
        })
    }

    pub(crate) async fn fetch_site_name(&self, client: &Client) -> Option<String> {
        match client.get_site_config("basic").await {
            Ok(cfg) => cfg.title,
            Err(_) => None,
        }
    }
}
