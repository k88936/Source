//! Mapping of SDK failures onto the application's error codes.
//!
//! Both halves of the remote layer talk to the same server, so the mapping
//! lives here instead of being duplicated per adapter.

use cloudreve_sdk_api::ApiError;

use crate::error::{AppError, ErrorCode};

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
