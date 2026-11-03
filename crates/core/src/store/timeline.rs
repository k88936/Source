//! Read paths over the whole index: the timeline and search. The contract lives
//! here, the statements it delegates to next to the asset table.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::*;

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

#[async_trait]
pub trait TimelineStore: Send + Sync {
    async fn timeline_page(
        &self,
        cursor: Option<&TimelineCursor>,
        limit: u32,
    ) -> Result<TimelinePage, AppError>;
    async fn search(&self, query: &SearchQuery) -> Result<TimelinePage, AppError>;
}

#[async_trait]
impl TimelineStore for SqliteCatalog {
    async fn timeline_page(
        &self,
        cursor: Option<&TimelineCursor>,
        limit: u32,
    ) -> Result<TimelinePage, AppError> {
        super::asset::timeline_page(self, cursor, limit).await
    }

    async fn search(&self, query: &SearchQuery) -> Result<TimelinePage, AppError> {
        super::asset::search(self, query).await
    }
}
