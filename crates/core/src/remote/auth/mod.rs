//! Signing in to a Cloudreve server: the contract the account use case calls,
//! and the two values a sign-in produces.
//!
//! Auth is the only stateful half of the remote layer. The token is never part
//! of the contract: it lives inside the SDK client, which storage gets to use
//! through [`CurrentAccount`].

use std::sync::Arc;

use async_trait::async_trait;
use cloudreve_sdk_api::Client;

use crate::error::AppError;

mod cloudreve;
mod dto;
mod login;
mod raw;

pub use cloudreve::CloudreveAuth;
pub use dto::{SignInOutcome, SignedIn, SiteInfo};

/// The contract the account use case signs in through. The Cloudreve
/// implementation is [`CloudreveAuth`]; `base_url` args are the account key.
#[async_trait]
pub trait RemoteAuth: Send + Sync {
    /// Probe a server before signing in: its name and thumbnail edge.
    async fn site_info(&self, base_url: &str) -> Result<SiteInfo, AppError>;

    async fn sign_in(
        &self,
        base_url: &str,
        email: &str,
        password: &str,
    ) -> Result<SignInOutcome, AppError>;

    async fn sign_in_2fa(
        &self,
        base_url: &str,
        session_id: &str,
        otp: &str,
    ) -> Result<SignedIn, AppError>;

    /// Restore the stored token of `base_url`, validating it against `/user/me`.
    async fn restore(&self, base_url: &str) -> Result<Option<SignedIn>, AppError>;

    async fn sign_out(&self, base_url: &str) -> Result<(), AppError>;
}

/// A signed-in client: the account's token is already installed on it, and
/// `base_url` is what its loopback-signed URLs are rewritten against.
#[derive(Clone)]
pub struct ActiveAccount {
    pub(crate) base_url: String,
    pub(crate) client: Arc<Client>,
}

/// The only thing storage needs from auth, and the reason it can depend on
/// auth without being able to reach the credentials.
pub trait CurrentAccount: Send + Sync {
    /// The live account; `Err(unauthorized)` when nobody is signed in.
    fn current(&self) -> Result<ActiveAccount, AppError>;
}
