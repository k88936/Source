//! Account payloads: the Cloudreve server a library lives on and the identity
//! signed in there.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// One Cloudreve account, keyed by its server.
///
/// The row is written as soon as a server is chosen, before the first login
/// succeeds, so `user` is absent until then.
///
/// Broader content is not stored here: see `crate::store::AlbumStore` for the
/// albums this account has.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UserInfo {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Account {
    pub base_url: String,
    pub user: Option<UserInfo>,
    pub site_name: Option<String>,
}
