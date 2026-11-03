//! Password / OTP login and session restore, built on the raw auth endpoints so
//! 2FA challenges survive the typed SDK layer.

use super::*;

impl CloudreveService {
    pub(crate) async fn authenticate(
        &self,
        base_url: &str,
        email: &str,
        password: &str,
    ) -> Result<LoginOutcome, AppError> {
        let base = base_url.trim_end_matches('/');
        let url = format!("{base}/api/v4/session/token");
        let envelope = raw_post::<LoginData>(
            &self.http,
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
                return Ok(LoginOutcome::TwoFactorRequired { session_id });
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
                .ok_or_else(|| AppError::internal("服务器要求 2FA，但没有返回会话 ID"))?;
            return Ok(LoginOutcome::TwoFactorRequired { session_id });
        }
        let session = self.adopt_tokens(base, data).await?;
        Ok(LoginOutcome::Authenticated { session })
    }

    pub(crate) async fn authenticate_otp(
        &self,
        base_url: &str,
        session_id: &str,
        otp: &str,
    ) -> Result<SessionState, AppError> {
        let base = base_url.trim_end_matches('/');
        let url = format!("{base}/api/v4/session/token/2fa");
        let envelope = raw_post::<LoginData>(
            &self.http,
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
    ) -> Result<Option<SessionState>, AppError> {
        let Some(raw) = self.store.read() else {
            return Ok(None);
        };
        let persisted: PersistedSession = match serde_json::from_str(&raw) {
            Ok(v) => v,
            Err(_) => {
                self.store.clear();
                return Ok(None);
            }
        };
        if persisted.base_url != base_url.trim_end_matches('/') {
            return Ok(None);
        }
        let client = self.client_for(&persisted.base_url);
        client
            .set_tokens_with_expiry(&persisted.token)
            .await
            .map_err(map_api_error)?;
        match client.get_user_me().await {
            Ok(user) => {
                let info = user_info(&user);
                let site_name = self.fetch_site_name(&client).await.or(persisted.site_name);
                self.install(&persisted.base_url, client, info, site_name, None)
                    .map(Some)
            }
            Err(ApiError::LoginRequired(_))
            | Err(ApiError::AccessTokenExpired)
            | Err(ApiError::RefreshTokenExpired) => {
                self.store.clear();
                Ok(None)
            }
            Err(err) => Err(map_api_error(err)),
        }
    }

    /// Store the tokens on `client`, sign the session in and persist it.
    async fn adopt_tokens(&self, base: &str, data: LoginData) -> Result<SessionState, AppError> {
        let client = self.client_for(base);
        client
            .set_tokens_with_expiry(&data.token)
            .await
            .map_err(map_api_error)?;
        let user = user_info(&data.user);
        let site_name = self.fetch_site_name(&client).await;
        self.install(base, client, user, site_name, Some(data.token))
    }
}
