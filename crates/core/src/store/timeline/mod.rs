//! Search over the whole index. The contract and the diesel statements behind
//! it.

use async_trait::async_trait;

use super::query::*;
use super::*;

mod dto;

pub use dto::*;

#[async_trait]
pub trait TimelineStore: Send + Sync {
    async fn search(&self, query: &SearchQuery) -> Result<TimelinePage, CatalogError>;
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
    async fn search(&self, query: &SearchQuery) -> Result<TimelinePage, CatalogError> {
        search(self, query).await
    }
}
