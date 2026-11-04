//! Password / OTP login and session restore, built on the raw auth endpoints so
//! 2FA challenges survive the typed SDK layer.

use cloudreve_sdk_api::ApiError;
use cloudreve_sdk_api::api::{SiteApi, UserApi};
use cloudreve_sdk_api::models::user::Token;

use super::cloudreve::CloudreveAuth;
use super::raw::{LoginData, raw_post, user_info};
use super::*;
use crate::remote::map_api_error;

impl CloudreveAuth {
    pub(crate) async fn authenticate(
        &self,
        base_url: &str,
        email: &str,
        password: &str,
    ) -> Result<SignInOutcome, AppError> {
        let base = base_url.trim_end_matches('/');
        let url = format!("{base}/api/v4/session/token");
        let envelope = raw_post::<LoginData>(
            self.http(),
            &url,
            &serde_json::json!({ "email": email, "password": password }),
        )
        .await?;

        if envelope.code != 0 {
            if (envelope.code == 203
                || envelope.code == 40005
                || envelope.msg.to_ascii_lowercase().contains("2fa"))
                && let Some(session_id) = envelope.data.as_ref().and_then(|d| d.session_id.clone())
            {
                return Ok(SignInOutcome::TwoFactorRequired { session_id });
            }
            return Err(AppError::unauthorized(if envelope.msg.is_empty() {
                "登录失败".to_string()
            } else {
                envelope.msg
            }));
        }

        let data = envelope
            .data
            .ok_or_else(|| AppError::internal("登录响应缺少数据"))?;
        if data.requires_2fa.unwrap_or(false) || data.two_fa.unwrap_or(false) {
            let session_id = data
                .session_id
                .ok_or_else(|| AppError::internal("服务器要求 2FA, 但没有返回会话 ID"))?;
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
    ) -> Result<SignedIn, AppError> {
        let base = base_url.trim_end_matches('/');
        let url = format!("{base}/api/v4/session/token/2fa");
        let envelope = raw_post::<LoginData>(
            self.http(),
            &url,
            &serde_json::json!({ "otp": otp, "session_id": session_id }),
        )
        .await?;
        if envelope.code != 0 {
            return Err(AppError::unauthorized(if envelope.msg.is_empty() {
                "验证码不正确".to_string()
            } else {
                envelope.msg
            }));
        }
        let data = envelope
            .data
            .ok_or_else(|| AppError::internal("2FA 响应缺少数据"))?;
        self.adopt_tokens(base, data).await
    }

    pub(crate) async fn restore_session(
        &self,
        base_url: &str,
    ) -> Result<Option<SignedIn>, AppError> {
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
            .map_err(map_api_error)?;
        match client.get_user_me().await {
            Ok(user) => {
                let user = user_info(&user);
                let site_name = self.site_name_of(&client).await;
                self.install(base, client);
                Ok(Some(SignedIn { user, site_name }))
            }
            Err(ApiError::LoginRequired(_))
            | Err(ApiError::AccessTokenExpired)
            | Err(ApiError::RefreshTokenExpired) => {
                self.clear_token(base);
                Ok(None)
            }
            Err(err) => Err(map_api_error(err)),
        }
    }

    /// Store the token on `client` and make it the live account.
    async fn adopt_tokens(&self, base: &str, data: LoginData) -> Result<SignedIn, AppError> {
        let client = self.client_for(base);
        client
            .set_tokens_with_expiry(&data.token)
            .await
            .map_err(map_api_error)?;
        let user = user_info(&data.user);
        let site_name = self.site_name_of(&client).await;
        self.persist(base, &data.token)?;
        self.install(base, client);
        Ok(SignedIn { user, site_name })
    }
}

#[async_trait]
impl RemoteAuth for CloudreveAuth {
    async fn site_info(&self, base_url: &str) -> Result<SiteInfo, AppError> {
        let client = self.client_for(base_url.trim_end_matches('/'));
        let cfg = client
            .get_site_config("basic")
            .await
            .map_err(map_api_error)?;
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
    ) -> Result<SignInOutcome, AppError> {
        self.authenticate(base_url, email, password).await
    }

    async fn sign_in_2fa(
        &self,
        base_url: &str,
        session_id: &str,
        otp: &str,
    ) -> Result<SignedIn, AppError> {
        self.authenticate_otp(base_url, session_id, otp).await
    }

    async fn restore(&self, base_url: &str) -> Result<Option<SignedIn>, AppError> {
        self.restore_session(base_url).await
    }

    async fn sign_out(&self, base_url: &str) -> Result<(), AppError> {
        if let Ok(active) = self.current() {
            active.client.clear_tokens().await;
        }
        *self.live().lock().unwrap() = None;
        self.clear_token(base_url.trim_end_matches('/'));
        Ok(())
    }
}
