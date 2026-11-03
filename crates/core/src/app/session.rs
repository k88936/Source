//! Session: server selection, login, 2FA and session restore.
//!
//! The signed-in session is owned here. Nothing else reads it directly, so the
//! library root it displays is re-derived from the catalog on every read instead
//! of being patched by whoever changes the root.

use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{AppError, ErrorCode};
use crate::remote::CloudrevePort;
use crate::store::{FolderRef, FolderStore, ServerStore, StoredServer};

use super::normalize_base_url;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub platform: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SiteInfo {
    pub name: String,
    pub thumbnail_size: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UserInfo {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Capacity {
    pub total: i64,
    pub used: i64,
    pub storage_pack_total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SessionState {
    pub server: String,
    pub site_name: Option<String>,
    pub user: UserInfo,
    pub library_root: Option<FolderRef>,
    pub capacity: Option<Capacity>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "kind"
)]
#[ts(export)]
#[allow(clippy::large_enum_variant)]
pub enum LoginOutcome {
    Authenticated { session: SessionState },
    TwoFactorRequired { session_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SetServerParams {
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LoginParams {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Login2faParams {
    pub session_id: String,
    pub otp: String,
}

pub struct SessionService<R, C> {
    cloud: R,
    catalog: C,
    pending_server: Mutex<Option<String>>,
    session: Mutex<Option<SessionState>>,
}

impl<R, C> SessionService<R, C> {
    pub fn new(cloud: R, catalog: C) -> Self {
        Self {
            cloud,
            catalog,
            pending_server: Mutex::new(None),
            session: Mutex::new(None),
        }
    }
}

impl<R, C> SessionService<R, C>
where
    R: CloudrevePort,
    C: FolderStore + ServerStore,
{
    pub async fn bootstrap(&self) -> Option<SessionState> {
        let server = self.catalog.load_server().await.ok().flatten()?;
        *self.pending_server.lock().unwrap() = Some(server.base_url.clone());
        match self.cloud.restore(&server.base_url).await {
            Ok(Some(mut session)) => {
                self.decorate_session(&mut session).await;
                self.cache_session(&session).await;
                *self.session.lock().unwrap() = Some(session.clone());
                Some(session)
            }
            Ok(None) => None,
            // 离线时仍让用户浏览已建立的 Catalog；真正的 401 会走 Ok(None)。
            Err(err) if matches!(err.code, ErrorCode::Network | ErrorCode::Server) => {
                let mut session = self
                    .catalog
                    .get_setting("session_snapshot")
                    .await
                    .ok()
                    .flatten()
                    .and_then(|raw| serde_json::from_str::<SessionState>(&raw).ok())?;
                session.server = server.base_url;
                self.decorate_session(&mut session).await;
                *self.session.lock().unwrap() = Some(session.clone());
                Some(session)
            }
            Err(_) => None,
        }
    }

    async fn cache_session(&self, session: &SessionState) {
        if let Ok(raw) = serde_json::to_string(session) {
            let _ = self.catalog.set_setting("session_snapshot", &raw).await;
        }
    }

    /// Base URL to talk to: the pending one set by the UI, else the last one
    /// remembered in the catalog.
    async fn current_base(&self, missing: &str) -> Result<String, AppError> {
        let pending = self.pending_server.lock().unwrap().clone();
        if let Some(base) = pending {
            return Ok(base);
        }
        self.catalog
            .get_setting("last_server")
            .await
            .ok()
            .flatten()
            .ok_or_else(|| AppError::internal(missing))
    }

    pub async fn set_server(&self, base_url: &str) -> Result<SiteInfo, AppError> {
        let base = normalize_base_url(base_url)?;
        let info = self.cloud.site_info(&base).await?;
        *self.pending_server.lock().unwrap() = Some(base.clone());
        self.catalog.set_setting("last_server", &base).await?;
        self.catalog.set_setting("site_name", &info.name).await?;
        Ok(info)
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<LoginOutcome, AppError> {
        let base = self.current_base("请先填写服务器地址").await?;
        let outcome = self.cloud.login(&base, email, password).await?;
        if let LoginOutcome::Authenticated { session } = &outcome {
            self.finalize_login(session.clone()).await?;
        }
        Ok(outcome)
    }

    pub async fn login_2fa(&self, session_id: &str, otp: &str) -> Result<SessionState, AppError> {
        let base = self.current_base("登录会话已过期，请重新输入密码").await?;
        let session = self.cloud.login_2fa(&base, session_id, otp).await?;
        self.finalize_login(session).await
    }

    async fn finalize_login(&self, mut session: SessionState) -> Result<SessionState, AppError> {
        self.decorate_session(&mut session).await;
        self.catalog
            .save_server(&StoredServer {
                base_url: session.server.clone(),
                user_id: session.user.id.clone(),
                display_name: Some(session.user.name.clone()),
            })
            .await?;
        self.catalog
            .set_setting("last_server", &session.server)
            .await?;
        if let Some(name) = &session.site_name {
            self.catalog.set_setting("site_name", name).await?;
        }
        self.cache_session(&session).await;
        *self.session.lock().unwrap() = Some(session.clone());
        Ok(session)
    }

    async fn decorate_session(&self, session: &mut SessionState) {
        session.library_root = self.catalog.library_root().await.ok().flatten();
        session.site_name = self
            .catalog
            .get_setting("site_name")
            .await
            .ok()
            .flatten()
            .or(session.site_name.take());
        if session.capacity.is_none() {
            session.capacity = self.cloud.capacity().await.ok();
        }
    }

    pub async fn logout(&self) -> Result<(), AppError> {
        let pending = self.pending_server.lock().unwrap().clone();
        let base = match pending {
            Some(base) => Some(base),
            None => self
                .catalog
                .load_server()
                .await
                .ok()
                .flatten()
                .map(|s| s.base_url),
        };
        if let Some(base) = base {
            let _ = self.cloud.logout(&base).await;
        }
        self.catalog.clear_server().await?;
        let _ = self.catalog.set_setting("session_snapshot", "").await;
        *self.session.lock().unwrap() = None;
        Ok(())
    }

    /// The signed-in session, with the library root read from the catalog so a
    /// root picked after login shows up without a second round trip.
    pub async fn session_state(&self) -> Option<SessionState> {
        let mut session = self.session.lock().unwrap().clone()?;
        session.library_root = self.catalog.library_root().await.ok().flatten();
        Some(session)
    }

    pub async fn refresh_capacity(&self) -> Result<Capacity, AppError> {
        let cap = self.cloud.capacity().await?;
        let session = {
            let mut guard = self.session.lock().unwrap();
            guard.as_mut().map(|session| {
                session.capacity = Some(cap.clone());
                session.clone()
            })
        };
        if let Some(session) = session {
            self.cache_session(&session).await;
        }
        Ok(cap)
    }
}
