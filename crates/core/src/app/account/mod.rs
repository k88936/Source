//! Account: server selection, sign-in, 2FA and the account snapshot.
//!
//! The account row is the authority. It holds the server and the identity, so
//! this service keeps no state of its own and every read is a query - that is
//! also why a restart, a logout or a second window all see the same account
//! without anything being patched in between.

mod dto;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::remote::{
    Capacity, CapacityError, ProbeError, RemoteAuth, RemoteStorage, SessionError, SignInError,
    SignInOutcome, SignedIn, SiteInfo,
};
use crate::store::{Account, AccountStore, CatalogError};

use super::normalize_base_url;

pub use dto::*;

/// What can go wrong while establishing or reading the account.
///
/// Each variant names the step that failed; the remote causes stay typed so the
/// shell can tell "the server refused the password" from "the address is not a
/// Cloudreve server" without reading a sentence.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum AccountError {
    /// The address typed in was empty or not a URL.
    ServerAddressInvalid { cause: super::UrlError },
    ProbeFailed { cause: ProbeError },
    SignInFailed { cause: SignInError },
    SessionFailed { cause: SessionError },
    CapacityFailed { cause: CapacityError },
    Catalog { cause: CatalogError },
    /// Signing in without a server chosen first.
    ServerMissing,
    /// A 2FA step whose original sign-in is gone.
    SignInMissing,
}

impl AccountError {
    fn catalog(cause: CatalogError) -> Self {
        AccountError::Catalog { cause }
    }
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
    /// Try the stored session once and say how it went.
    ///
    /// The stored identity is what the UI shows when the server cannot be
    /// reached: a refused session drops it, but a network failure does not,
    /// because an offline library is still browsable.
    ///
    /// This is the one call of a start that waits for a server, so the shell
    /// runs it off the main thread and opens the window from the row on disk.
    pub async fn restore(&self) -> Result<RestoreOutcome, AccountError> {
        self.restore_impl().await
    }

    /// Whether a signed-in client stands behind the account right now.
    pub fn is_live(&self) -> bool {
        self.auth.is_live()
    }

    /// Remember the server before the first sign-in, keeping the identity of
    /// an account that used this server before.
    ///
    /// Switching to a different server drops the catalog: albums, their assets
    /// and the local album mapping all point at folders of the server the
    /// current account lives on.
    pub async fn set_server(&self, base_url: &str) -> Result<SiteInfo, AccountError> {
        self.set_server_impl(base_url).await
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<LoginOutcome, AccountError> {
        self.login_impl(email, password).await
    }

    pub async fn login_2fa(&self, session_id: &str, otp: &str) -> Result<AccountState, AccountError> {
        self.login_2fa_impl(session_id, otp).await
    }

    pub async fn logout(&self) -> Result<(), AccountError> {
        self.logout_impl().await
    }

    pub async fn get_account_state(&self) -> Result<Option<AccountState>, AccountError> {
        self.get_account_state_impl().await
    }

    pub async fn get_capacity(&self) -> Result<Capacity, AccountError> {
        self.get_capacity_impl().await
    }
}

// ---------------------------------------------------------------------
// Use case implementations
// ---------------------------------------------------------------------

impl<A, S, C> AccountService<A, S, C>
where
    A: RemoteAuth,
    S: RemoteStorage,
    C: AccountStore,
{
    async fn restore_impl(&self) -> Result<RestoreOutcome, AccountError> {
        let Some(account) = self.current_account().await? else {
            return Ok(RestoreOutcome::Empty);
        };
        match self.auth.restore(&account.base_url).await {
            Ok(Some(signed_in)) => Ok(RestoreOutcome::Restored {
                account: self.remember(&account, signed_in).await?,
            }),
            Ok(None) => {
                self.forget_user(account).await?;
                Ok(RestoreOutcome::Rejected)
            }
            // The token was not refused - `restore` reports that as `Ok(None)`
            // - so the identity on disk is still the best answer this start
            // can give, and an offline library stays browsable.
            Err(_) => Ok(RestoreOutcome::Unreachable {
                account: account.user.map(|user| AccountState {
                    server: account.base_url,
                    site_name: account.site_name,
                    user,
                }),
            }),
        }
    }

    async fn set_server_impl(&self, base_url: &str) -> Result<SiteInfo, AccountError> {
        let base = normalize_base_url(base_url)
            .map_err(|cause| AccountError::ServerAddressInvalid { cause })?;
        let info = self
            .auth
            .site_info(&base)
            .await
            .map_err(|cause| AccountError::ProbeFailed { cause })?;
        let known = self
            .account_store
            .get_account(&base)
            .await
            .map_err(AccountError::catalog)?;
        if let Some(previous) = self
            .account_store
            .get_current_account()
            .await
            .map_err(AccountError::catalog)?
            && previous.base_url != base
        {
            self.account_store
                .reset_catalog()
                .await
                .map_err(AccountError::catalog)?;
        }
        self.account_store
            .save_account(&Account {
                base_url: base,
                user: known.as_ref().and_then(|account| account.user.clone()),
                site_name: Some(info.name.clone()),
            })
            .await
            .map_err(AccountError::catalog)?;
        Ok(info)
    }

    async fn login_impl(&self, email: &str, password: &str) -> Result<LoginOutcome, AccountError> {
        let account = self.require_server().await?;
        match self
            .auth
            .sign_in(&account.base_url, email, password)
            .await
            .map_err(|cause| AccountError::SignInFailed { cause })?
        {
            SignInOutcome::SignedIn(signed_in) => Ok(LoginOutcome::Authenticated {
                account: self.remember(&account, signed_in).await?,
            }),
            SignInOutcome::TwoFactorRequired { session_id } => {
                Ok(LoginOutcome::TwoFactorRequired { session_id })
            }
        }
    }

    async fn login_2fa_impl(&self, session_id: &str, otp: &str) -> Result<AccountState, AccountError> {
        let account = self.require_session().await?;
        let signed_in = self
            .auth
            .sign_in_2fa(&account.base_url, session_id, otp)
            .await
            .map_err(|cause| AccountError::SignInFailed { cause })?;
        self.remember(&account, signed_in).await
    }

    async fn logout_impl(&self) -> Result<(), AccountError> {
        let Some(account) = self
            .account_store
            .get_current_account()
            .await
            .map_err(AccountError::catalog)?
        else {
            return Ok(());
        };
        let _ = self.auth.sign_out(&account.base_url).await;
        // The row keeps the server so the next sign-in starts from the same
        // place; only the identity is dropped.
        self.forget_user(account).await?;
        Ok(())
    }

    async fn get_account_state_impl(&self) -> Result<Option<AccountState>, AccountError> {
        let Some(account) = self
            .account_store
            .get_current_account()
            .await
            .map_err(AccountError::catalog)?
        else {
            return Ok(None);
        };
        Ok(account.user.map(|user| AccountState {
            server: account.base_url,
            site_name: account.site_name,
            user,
        }))
    }

    async fn get_capacity_impl(&self) -> Result<Capacity, AccountError> {
        self.storage
            .get_capacity()
            .await
            .map_err(|cause| AccountError::CapacityFailed { cause })
    }

    async fn require_server(&self) -> Result<Account, AccountError> {
        self.current_account()
            .await?
            .ok_or(AccountError::ServerMissing)
    }

    async fn require_session(&self) -> Result<Account, AccountError> {
        self.current_account()
            .await?
            .ok_or(AccountError::SignInMissing)
    }

    async fn current_account(&self) -> Result<Option<Account>, AccountError> {
        self.account_store
            .get_current_account()
            .await
            .map_err(AccountError::catalog)
    }

    async fn remember(
        &self,
        account: &Account,
        signed_in: SignedIn,
    ) -> Result<AccountState, AccountError> {
        let saved = Account {
            base_url: account.base_url.clone(),
            user: Some(signed_in.user.clone()),
            site_name: signed_in.site_name.or_else(|| account.site_name.clone()),
        };
        self.account_store
            .save_account(&saved)
            .await
            .map_err(AccountError::catalog)?;
        Ok(AccountState {
            server: saved.base_url,
            site_name: saved.site_name,
            user: signed_in.user,
        })
    }

    async fn forget_user(&self, account: Account) -> Result<Account, AccountError> {
        let cleared = Account {
            base_url: account.base_url,
            user: None,
            site_name: account.site_name,
        };
        self.account_store
            .save_account(&cleared)
            .await
            .map_err(AccountError::catalog)?;
        Ok(cleared)
    }
}
