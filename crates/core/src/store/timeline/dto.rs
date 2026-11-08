//! Timeline and search payloads: one page, its cursor, and the query the UI
//! sends.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::super::asset::{AssetSummary, MediaType};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TimelineCursor {
    pub taken_at: i64,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TimelinePage {
    pub items: Vec<AssetSummary>,
    pub next: Option<TimelineCursor>,
    pub total: u32,
}

#[derive(Debug, Clone, Default)]
pub struct SearchQuery {
    pub query: String,
    pub from: Option<i64>,
    pub to: Option<i64>,
    pub favorite: Option<bool>,
    pub media_type: Option<MediaType>,
    pub cursor: Option<TimelineCursor>,
    pub limit: u32,
}
