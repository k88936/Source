//! Raw Cloudreve auth envelopes and HTTP error mapping.

use super::*;

// ---------------------------------------------------------------------------
// Raw auth envelope
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub(crate) struct Envelope<T> {
    #[serde(default)]
    pub(crate) code: i64,
    #[serde(default)]
    pub(crate) msg: String,
    pub(crate) data: Option<T>,
}

#[derive(Deserialize)]
pub(crate) struct LoginData {
    pub(crate) user: cloudreve_sdk_api::models::user::User,
    pub(crate) token: Token,
    #[serde(default)]
    pub(crate) session_id: Option<String>,
    #[serde(default)]
    pub(crate) requires_2fa: Option<bool>,
    #[serde(default)]
    pub(crate) two_fa: Option<bool>,
}

pub(crate) async fn raw_post<T: serde::de::DeserializeOwned>(
    http: &reqwest::Client,
    url: &str,
    body: &serde_json::Value,
) -> Result<Envelope<T>, AppError> {
    let response = http
        .post(url)
        .json(body)
        .send()
        .await
        .map_err(|e| AppError::network(format!("连接服务器失败: {e}")))?;
    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|e| AppError::network(format!("读取服务器响应失败: {e}")))?;
    if status.as_u16() == 401 || status.as_u16() == 403 {
        return Err(AppError::unauthorized("邮箱或密码错误"));
    }
    serde_json::from_str::<Envelope<T>>(&text).map_err(|e| {
        AppError::new(
            ErrorCode::Server,
            format!("服务器返回了无法识别的响应 (HTTP {status}): {e}"),
            status.is_server_error(),
        )
    })
}

pub(crate) fn user_info(user: &cloudreve_sdk_api::models::user::User) -> UserInfo {
    UserInfo {
        id: user.id.clone(),
        name: if user.nickname.is_empty() {
            user.email.clone().unwrap_or_else(|| user.id.clone())
        } else {
            user.nickname.clone()
        },
        email: user.email.clone(),
    }
}

pub(crate) fn map_api_error(err: ApiError) -> AppError {
    match err {
        ApiError::LoginRequired(message) => AppError::unauthorized(message),
        ApiError::AccessTokenExpired
        | ApiError::RefreshTokenExpired
        | ApiError::NoTokensAvailable => AppError::unauthorized("登录已过期"),
        ApiError::ApiError { code, message, .. } => match code {
            401 | 40020 | 40089 => AppError::unauthorized(message),
            403 => AppError::permission(message),
            404 => AppError::not_found(message),
            40004 => AppError::conflict(message),
            40073 => AppError::conflict(message),
            40081 => AppError::new(ErrorCode::Server, message, true),
            _ => AppError::new(ErrorCode::Server, message, code >= 500),
        },
        ApiError::LockConflict { message, .. } => AppError::conflict(message),
        ApiError::BatchError { message, .. } => AppError::new(ErrorCode::Server, message, true),
        ApiError::RequestError(e) => AppError::network(format!("网络请求失败: {e}")),
        ApiError::JsonError(e) => AppError::internal(format!("解析响应失败: {e}")),
        ApiError::InvalidUrl(e) => AppError::internal(format!("服务器地址无效: {e}")),
        ApiError::InvalidToken(e) => AppError::unauthorized(e),
        ApiError::Other(e) => AppError::internal(e),
        other => AppError::internal(other.to_string()),
    }
}
