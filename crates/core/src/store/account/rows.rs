//! Row struct of the `account` table, plus the JSON columns it decodes.

use diesel::Queryable;
use serde::Serialize;
use serde::de::DeserializeOwned;

use super::dto::Account;
use super::*;

#[derive(Queryable)]
pub(crate) struct AccountRow {
    pub(crate) base_url: String,
    pub(crate) user: Option<String>,
    pub(crate) site_name: Option<String>,
    pub(crate) library_root: Option<String>,
}

impl AccountRow {
    pub(crate) fn into_account(self) -> Result<Account, AppError> {
        Ok(Account {
            base_url: self.base_url,
            user: decode(self.user, "用户")?,
            site_name: self.site_name,
            library_root: decode(self.library_root, "图库根")?,
        })
    }
}

/// The user and the library root are single values, so each is one JSON column
/// instead of a set of columns that could disagree with each other.
pub(crate) fn encode<T: Serialize>(value: &Option<T>) -> Result<Option<String>, AppError> {
    value
        .as_ref()
        .map(|value| {
            serde_json::to_string(value)
                .map_err(|error| AppError::internal(format!("无法序列化账号字段: {error}")))
        })
        .transpose()
}

fn decode<T: DeserializeOwned>(raw: Option<String>, what: &str) -> Result<Option<T>, AppError> {
    raw.map(|raw| {
        serde_json::from_str(&raw)
            .map_err(|error| AppError::internal(format!("无法解析账号{what}: {error}")))
    })
    .transpose()
}
