//! Read paths over the whole index: the timeline and search. The contract and
//! the diesel statements behind it.

use async_trait::async_trait;

use super::query::*;
use super::*;

mod dto;

pub use dto::*;

#[async_trait]
pub trait TimelineStore: Send + Sync {
    async fn get_timeline_page(
        &self,
        cursor: Option<&TimelineCursor>,
        limit: u32,
    ) -> Result<TimelinePage, CatalogError>;
    async fn search(&self, query: &SearchQuery) -> Result<TimelinePage, CatalogError>;
}

pub(crate) async fn get_timeline_page(
    cat: &SqliteCatalog,
    cursor: Option<&TimelineCursor>,
    limit: u32,
) -> Result<TimelinePage, CatalogError> {
    let mut conn = cat.conn().await?;
    asset_page(&mut conn, &AssetFilter::default(), cursor, limit).await
}

pub(crate) async fn search(
    cat: &SqliteCatalog,
    query: &SearchQuery,
) -> Result<TimelinePage, CatalogError> {
    let mut conn = cat.conn().await?;
    let needle = query.query.trim();
    let filter = AssetFilter {
        folder: query.folder.as_ref().map(|id| id.as_str()),
        name: (!needle.is_empty()).then_some(needle),
        from: query.from,
        to: query.to,
        favorite: query.favorite,
        media_type: query.media_type,
    };
    asset_page(&mut conn, &filter, query.cursor.as_ref(), query.limit).await
}

#[async_trait]
impl TimelineStore for SqliteCatalog {
    async fn get_timeline_page(
        &self,
        cursor: Option<&TimelineCursor>,
        limit: u32,
    ) -> Result<TimelinePage, CatalogError> {
        get_timeline_page(self, cursor, limit).await
    }

    async fn search(&self, query: &SearchQuery) -> Result<TimelinePage, CatalogError> {
        search(self, query).await
    }
}
