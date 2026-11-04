//! Signing in with the credentials the examples read from the environment.

use std::sync::Arc;

use anyhow::{Result, anyhow};
use source_core::{CloudreveAuth, RemoteAuth, SignInOutcome};

use super::env::Settings;
use super::mock::credential_store::MemoryCredentials;

/// Sign in with the credentials from [`Settings`], returning the auth that
/// storage reads its signed-in client from.
pub async fn sign_in(settings: &Settings) -> Result<CloudreveAuth> {
    let auth = CloudreveAuth::new(Arc::new(MemoryCredentials::default()));
    let outcome = auth
        .sign_in(&settings.server, &settings.email, &settings.password)
        .await?;
    match outcome {
        SignInOutcome::SignedIn(_) => Ok(auth),
        SignInOutcome::TwoFactorRequired { .. } => {
            Err(anyhow!("该账号要求两步验证, 示例无法自动登录"))
        }
    }
}
