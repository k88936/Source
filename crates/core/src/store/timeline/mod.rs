//! Read paths over the whole index: the timeline and search. The contract lives
//! here, the statements it delegates to next to the asset table.

use async_trait::async_trait;

use super::*;

mod dto;

pub use dto::*;

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
