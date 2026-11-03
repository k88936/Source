//! What a sign-in produces.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::store::UserInfo;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SiteInfo {
    pub name: String,
    pub thumbnail_size: Option<u32>,
}

/// A token that the server accepted: who it belongs to, and the name the
/// server calls itself.
///
/// It carries no `base_url`, because the account it is stored under *is* the
/// base URL.
#[derive(Debug, Clone)]
pub struct SignedIn {
    pub user: UserInfo,
    pub site_name: Option<String>,
}

#[derive(Debug, Clone)]
pub enum SignInOutcome {
    SignedIn(SignedIn),
    TwoFactorRequired { session_id: String },
}
