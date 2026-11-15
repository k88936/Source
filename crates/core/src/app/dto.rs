//! What the running build reports about itself.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// What `app_info` reports about the running build.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub platform: String,
}
