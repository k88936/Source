//! Account: server selection, sign-in, 2FA and the account snapshot.
//!
//! The account row is the authority. It holds the server and the identity, so
//! this service keeps no state of its own and every read is a query - that is
//! also why a restart, a logout or a second window all see the same account
//! without anything being patched in between.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{AppError, ErrorCode};
use crate::remote::{Capacity, RemoteAuth, RemoteStorage, SignInOutcome, SignedIn, SiteInfo};
use crate::store::{Account, AccountStore, UserInfo};

use super::normalize_base_url;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AccountState {
    pub server: String,
    pub site_name: Option<String>,
    pub user: UserInfo,
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
    Authenticated { account: AccountState },
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

pub struct AccountService<A, S, C> {
    auth: A,
    storage: S,
    account_store: C,
}

impl<A, S, C> AccountService<A, S, C> {
    pub fn new(auth: A, storage: S, catalog: C) -> Self {
        Self {
            auth,
            storage,
            account_store: catalog,
        }
    }
}

impl<A, S, C> AccountService<A, S, C>
where
    A: RemoteAuth,
    S: RemoteStorage,
    C: AccountStore,
{
    /// Restore the account the app was last used with.
    ///
    /// The stored identity is what the UI shows when the server cannot be
    /// reached: a dead token drops it, but a network failure does not, because
    /// an offline library is still browsable.
    pub async fn bootstrap(&self) -> Option<AccountState> {
        let account = self
            .account_store
            .current_account()
            .await
            .ok()
            .flatten()?;
        match self.auth.restore(&account.base_url).await {
            Ok(Some(signed_in)) => self.remember(&account, signed_in).await.ok(),
            Ok(None) => {
                self.forget_user(account).await.ok()?;
                None
            }
            Err(err) if matches!(err.code, ErrorCode::Network | ErrorCode::Server) => {
                let user = account.user?;
                Some(AccountState {
                    server: account.base_url,
                    site_name: account.site_name,
                    user,
                })
            }
            Err(_) => None,
        }
    }

    /// Remember the server before the first sign-in, keeping the identity of
    /// an account that used this server before.
    ///
    /// Switching to a different server drops the catalog: albums, their assets
    /// and the local album mapping all point at folders of the server the
    /// current account lives on.
    pub async fn set_server(&self, base_url: &str) -> Result<SiteInfo, AppError> {
        let base = normalize_base_url(base_url)?;
        let info = self.auth.site_info(&base).await?;
        let known = self.account_store.account(&base).await?;
        if let Some(previous) = self.account_store.current_account().await?
            && previous.base_url != base
        {
            self.account_store.reset_catalog().await?;
        }
        self.account_store
            .save_account(&Account {
                base_url: base,
                user: known.as_ref().and_then(|account| account.user.clone()),
                site_name: Some(info.name.clone()),
            })
            .await?;
        Ok(info)
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<LoginOutcome, AppError> {
        let account = self.current_account("请先填写服务器地址").await?;
        match self
            .auth
            .sign_in(&account.base_url, email, password)
            .await?
        {
            SignInOutcome::SignedIn(signed_in) => Ok(LoginOutcome::Authenticated {
                account: self.remember(&account, signed_in).await?,
            }),
            SignInOutcome::TwoFactorRequired { session_id } => {
                Ok(LoginOutcome::TwoFactorRequired { session_id })
            }
        }
    }

    pub async fn login_2fa(&self, session_id: &str, otp: &str) -> Result<AccountState, AppError> {
        let account = self
            .current_account("登录会话已过期, 请重新输入密码")
            .await?;
        let signed_in = self
            .auth
            .sign_in_2fa(&account.base_url, session_id, otp)
            .await?;
        self.remember(&account, signed_in).await
    }

    pub async fn logout(&self) -> Result<(), AppError> {
        let Some(account) = self.account_store.current_account().await? else {
            return Ok(());
        };
        let _ = self.auth.sign_out(&account.base_url).await;
        // The row keeps the server so the next sign-in starts from the same
        // place; only the identity is dropped.
        self.forget_user(account).await?;
        Ok(())
    }

    pub async fn account_state(&self) -> Result<Option<AccountState>, AppError> {
        let Some(account) = self.account_store.current_account().await? else {
            return Ok(None);
        };
        Ok(account.user.map(|user| AccountState {
            server: account.base_url,
            site_name: account.site_name,
            user,
        }))
    }

    pub async fn capacity(&self) -> Result<Capacity, AppError> {
        self.storage.capacity().await
    }

    async fn current_account(&self, missing: &str) -> Result<Account, AppError> {
        self.account_store
            .current_account()
            .await?
            .ok_or_else(|| AppError::internal(missing))
    }

    async fn remember(
        &self,
        account: &Account,
        signed_in: SignedIn,
    ) -> Result<AccountState, AppError> {
        let saved = Account {
            base_url: account.base_url.clone(),
            user: Some(signed_in.user.clone()),
            site_name: signed_in.site_name.or_else(|| account.site_name.clone()),
        };
        self.account_store.save_account(&saved).await?;
        Ok(AccountState {
            server: saved.base_url,
            site_name: saved.site_name,
            user: signed_in.user,
        })
    }

    async fn forget_user(&self, account: Account) -> Result<Account, AppError> {
        let cleared = Account {
            base_url: account.base_url,
            user: None,
            site_name: account.site_name,
        };
        self.account_store.save_account(&cleared).await?;
        Ok(cleared)
    }
}
