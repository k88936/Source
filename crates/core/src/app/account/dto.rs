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

/// How far restoring the stored session has got.
///
/// The page opens on the account row from sqlite; this says whether the client
/// behind that row is already usable, still being checked, or gave up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum SessionSetup {
    /// The stored session is being checked; the answer is not in yet.
    Restoring,
    /// A signed-in client stands behind the account.
    Live,
    /// The check is over and no client stands behind the account: the server
    /// did not answer, or a sign-out left the row with nothing to check.
    Offline,
}

/// The account row and what the client behind it can do right now.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SessionState {
    pub account: Option<AccountState>,
    pub setup: SessionSetup,
}

/// What one try at the stored session produced.
///
/// The shell keeps the loop: how often an unanswered server is worth another
/// visit belongs to the start, not to the use case.
#[derive(Debug, Clone)]
pub enum RestoreOutcome {
    /// No account row: there is nothing to restore.
    Empty,
    /// The stored session checks out; the row now carries the refreshed name.
    Restored { account: AccountState },
    /// The server refused the session, so the row lost its identity.
    Rejected,
    /// The server did not answer; `account` is the row as it stands.
    Unreachable { account: Option<AccountState> },
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
