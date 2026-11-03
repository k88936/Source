//! Account payloads: the Cloudreve server a library lives on, the identity
//! signed in there, and that account's library root.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::store::FolderRef;

/// One Cloudreve account, keyed by its server.
///
/// The row is written as soon as a server is chosen, before the first login
/// succeeds, so `user` is absent until then. `library_root` belongs to the
/// account rather than to the catalog as a whole: the tree it points at is the
/// one this account indexed.
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
    pub library_root: Option<FolderRef>,
}
