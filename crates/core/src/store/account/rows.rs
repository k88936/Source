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
}

impl AccountRow {
    pub(crate) fn into_account(self) -> Result<Account, CatalogError> {
        Ok(Account {
            base_url: self.base_url,
            user: decode(self.user, "用户")?,
            site_name: self.site_name,
        })
    }
}

/// The user is a single value, so it is one JSON column instead of a set of
/// columns that could disagree with each other.
pub(crate) fn encode<T: Serialize>(value: &Option<T>) -> Result<Option<String>, CatalogError> {
    value
        .as_ref()
        .map(|value| {
            serde_json::to_string(value)
                .map_err(|error| CatalogError::EncodeFailed {
                    cause: format!("无法序列化账号字段: {error}"),
                })
        })
        .transpose()
}

fn decode<T: DeserializeOwned>(raw: Option<String>, what: &str) -> Result<Option<T>, CatalogError> {
    raw.map(|raw| {
        serde_json::from_str(&raw)
            .map_err(|error| CatalogError::DecodeFailed {
                cause: format!("无法解析账号{what}: {error}"),
            })
    })
    .transpose()
}
