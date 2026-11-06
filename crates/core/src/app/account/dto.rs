//! Parameters the account use cases accept, the state they report, and the way
//! a sign-in ends.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::store::UserInfo;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AccountState {
    pub server: String,
    pub site_name: Option<String>,
    pub user: UserInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "kind"
)]
#[ts(export)]
#[allow(clippy::large_enum_variant)]
pub enum LoginOutcome {
    Authenticated { account: AccountState },
    TwoFactorRequired { session_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SetServerParams {
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LoginParams {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Login2faParams {
    pub session_id: String,
    pub otp: String,
}
