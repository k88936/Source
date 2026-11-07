//! Password / OTP login and session restore, built on the raw auth endpoints so
//! 2FA challenges survive the typed SDK layer.

use cloudreve_sdk_api::api::{SiteApi, UserApi};
use cloudreve_sdk_api::models::user::Token;

use super::cloudreve::CloudreveAuth;
use super::raw::{LoginData, raw_post, user_info};
use super::*;
use crate::remote::map_api_error;

/// The credential step of a password sign-in.
fn sign_in_error(cause: RemoteError) -> SignInError {
    match cause {
        RemoteError::Unreachable { cause } => SignInError::Unreachable { cause },
        RemoteError::SessionExpired { message } => SignInError::CredentialsRejected { message },
        RemoteError::NotSignedIn => SignInError::CredentialsRejected { message: None },
        RemoteError::Rejected { status, .. } if matches!(status, 401 | 403 | 40020) => {
            SignInError::CredentialsRejected { message: None }
        }
        RemoteError::Rejected { status, message } => SignInError::Rejected { status, message },
        RemoteError::Protocol { cause } => SignInError::Protocol { cause },
    }
}

/// The one-time-code step of a sign-in.
fn code_error(cause: RemoteError) -> SignInError {
    match cause {
        RemoteError::Unreachable { cause } => SignInError::Unreachable { cause },
        RemoteError::SessionExpired { message } => SignInError::CodeRejected { message },
        RemoteError::NotSignedIn => SignInError::CodeRejected { message: None },
        RemoteError::Rejected { status, .. } if matches!(status, 401 | 403 | 40020) => {
            SignInError::CodeRejected { message: None }
        }
        RemoteError::Rejected { status, message } => SignInError::Rejected { status, message },
        RemoteError::Protocol { cause } => SignInError::Protocol { cause },
    }
}

/// Restoring: a transport failure is a failed check, not a refused session.
fn session_error(cause: RemoteError) -> SessionError {
    match cause {
        RemoteError::Unreachable { cause } => SessionError::Unreachable { cause },
        RemoteError::SessionExpired { message } => SessionError::Rejected {
            status: 401,
            message: message.unwrap_or_default(),
        },
        RemoteError::NotSignedIn => SessionError::Rejected {
            status: 401,
            message: String::new(),
        },
        RemoteError::Rejected { status, message } => SessionError::Rejected { status, message },
        RemoteError::Protocol { cause } => SessionError::Protocol { cause },
    }
}

/// Whether the stored credential is dead rather than temporarily unusable.
fn session_is_gone(cause: &RemoteError) -> bool {
    matches!(
        cause,
        RemoteError::NotSignedIn
            | RemoteError::SessionExpired { .. }
            | RemoteError::Rejected {
                status: 401 | 403,
                ..
            }
    )
}

impl CloudreveAuth {
    pub(crate) async fn authenticate(
        &self,
        base_url: &str,
        email: &str,
        password: &str,
    ) -> Result<SignInOutcome, SignInError> {
        let base = base_url.trim_end_matches('/');
        let url = format!("{base}/api/v4/session/token");
        let envelope = raw_post::<LoginData>(
            self.http(),
            &url,
            &serde_json::json!({ "email": email, "password": password }),
        )
        .await
        .map_err(sign_in_error)?;

        if envelope.code != 0 {
            if (envelope.code == 203
                || envelope.code == 40005
                || envelope.msg.to_ascii_lowercase().contains("2fa"))
                && let Some(session_id) = envelope.data.as_ref().and_then(|d| d.session_id.clone())
            {
                return Ok(SignInOutcome::TwoFactorRequired { session_id });
            }
            return Err(SignInError::CredentialsRejected {
                message: (!envelope.msg.is_empty()).then_some(envelope.msg),
            });
        }

        let data = envelope.data.ok_or(SignInError::Protocol {
            cause: "登录响应缺少数据".to_string(),
        })?;
        if data.requires_2fa.unwrap_or(false) || data.two_fa.unwrap_or(false) {
            let session_id = data.session_id.ok_or(SignInError::Protocol {
                cause: "服务器要求 2FA, 但没有返回会话 ID".to_string(),
            })?;
            return Ok(SignInOutcome::TwoFactorRequired { session_id });
        }
        Ok(SignInOutcome::SignedIn(
            self.adopt_tokens(base, data).await?,
        ))
    }

    pub(crate) async fn authenticate_otp(
        &self,
        base_url: &str,
        session_id: &str,
        otp: &str,
    ) -> Result<SignedIn, SignInError> {
        let base = base_url.trim_end_matches('/');
        let url = format!("{base}/api/v4/session/token/2fa");
        let envelope = raw_post::<LoginData>(
            self.http(),
            &url,
            &serde_json::json!({ "otp": otp, "session_id": session_id }),
        )
        .await
        .map_err(code_error)?;
        if envelope.code != 0 {
            return Err(SignInError::CodeRejected {
                message: (!envelope.msg.is_empty()).then_some(envelope.msg),
            });
        }
        let data = envelope.data.ok_or(SignInError::Protocol {
            cause: "2FA 响应缺少数据".to_string(),
        })?;
        self.adopt_tokens(base, data).await
    }

    pub(crate) async fn restore_session(
        &self,
        base_url: &str,
    ) -> Result<Option<SignedIn>, SessionError> {
        let base = base_url.trim_end_matches('/');
        let Some(raw) = self.read_token(base) else {
            return Ok(None);
        };
        let token: Token = match serde_json::from_str(&raw) {
            Ok(token) => token,
            // A credential this build cannot read is no credential at all.
            Err(_) => {
                self.clear_token(base);
                return Ok(None);
            }
        };
        let client = self.client_for(base);
        client
            .set_tokens_with_expiry(&token)
            .await
            .map_err(map_api_error)
            .map_err(session_error)?;
        match client.get_user_me().await.map_err(map_api_error) {
            Ok(user) => {
                let user = user_info(&user);
                let site_name = self.site_name_of(&client).await;
                self.install(base, Arc::clone(&client));
                Ok(Some(SignedIn { user, site_name }))
            }
            Err(cause) if session_is_gone(&cause) => {
                self.clear_token(base);
                Ok(None)
            }
            Err(cause) => Err(session_error(cause)),
        }
    }

    /// Store the token on `client` and make it the live account.
    async fn adopt_tokens(&self, base: &str, data: LoginData) -> Result<SignedIn, SignInError> {
        let client = self.client_for(base);
        client
            .set_tokens_with_expiry(&data.token)
            .await
            .map_err(map_api_error)
            .map_err(sign_in_error)?;
        let user = user_info(&data.user);
        let site_name = self.site_name_of(&client).await;
        self.persist(base, &data.token).map_err(sign_in_error)?;
        self.install(base, client);
        Ok(SignedIn { user, site_name })
    }
}

#[async_trait]
impl RemoteAuth for CloudreveAuth {
    async fn site_info(&self, base_url: &str) -> Result<SiteInfo, ProbeError> {
        let client = self.client_for(base_url.trim_end_matches('/'));
        let cfg = client
            .get_site_config("basic")
            .await
            .map_err(|cause| probe_error(map_api_error(cause)))?;
        let thumbnail_size = client
            .get_site_config("thumb")
            .await
            .ok()
            .and_then(|c| c.thumbnail_width)
            .filter(|v| *v > 0)
            .map(|v| v as u32);
        Ok(SiteInfo {
            name: cfg.title.unwrap_or_else(|| "Cloudreve".to_string()),
            thumbnail_size,
        })
    }

    async fn sign_in(
        &self,
        base_url: &str,
        email: &str,
        password: &str,
    ) -> Result<SignInOutcome, SignInError> {
        self.authenticate(base_url, email, password).await
    }

    async fn sign_in_2fa(
        &self,
        base_url: &str,
        session_id: &str,
        otp: &str,
    ) -> Result<SignedIn, SignInError> {
        self.authenticate_otp(base_url, session_id, otp).await
    }

    async fn restore(&self, base_url: &str) -> Result<Option<SignedIn>, SessionError> {
        self.restore_session(base_url).await
    }

    async fn sign_out(&self, base_url: &str) -> Result<(), SessionError> {
        if let Ok(active) = self.current() {
            active.client.clear_tokens().await;
        }
        *self.live().lock().unwrap() = None;
        self.clear_token(base_url.trim_end_matches('/'));
        Ok(())
    }
}

/// Probing a server: an answer we cannot parse means the address is not a
/// Cloudreve server, which is a different sentence than "unreachable".
fn probe_error(cause: RemoteError) -> ProbeError {
    match cause {
        RemoteError::Unreachable { cause } => ProbeError::Unreachable { cause },
        RemoteError::Protocol { cause } => ProbeError::NotCloudreve { detail: cause },
        RemoteError::Rejected { status, message } => ProbeError::Rejected { status, message },
        RemoteError::SessionExpired { message } => ProbeError::NotCloudreve {
            detail: message.unwrap_or_default(),
        },
        RemoteError::NotSignedIn => ProbeError::NotCloudreve {
            detail: "服务器要求登录".to_string(),
        },
    }
}
