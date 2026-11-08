//! Read paths over the whole index: the timeline and search. The contract and
//! the diesel statements behind it.

use async_trait::async_trait;

use super::asset::rows::AssetRow;
use super::asset::schema::assets;
use super::asset::values::media_type_to_str;
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
    timeline(&mut conn, None, cursor, limit).await
}

pub(crate) async fn search(
    cat: &SqliteCatalog,
    query: &SearchQuery,
) -> Result<TimelinePage, CatalogError> {
    let mut conn = cat.conn().await?;
    let limit = i64::from(query.limit.clamp(1, 200));
    let mut statement = assets::table
        .filter(assets::media_type.eq_any(MEDIA_TYPES))
        .select(AssetRow::as_select())
        .into_boxed::<Sqlite>();
    let mut total_query = assets::table
        .filter(assets::media_type.eq_any(MEDIA_TYPES))
        .select(diesel::dsl::count_star())
        .into_boxed::<Sqlite>();
    let needle = query.query.trim();
    if !needle.is_empty() {
        let escaped = needle
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_");
        let pattern = format!("%{escaped}%");
        statement = statement.filter(assets::name.like(pattern.clone()).escape('\\'));
        total_query = total_query.filter(assets::name.like(pattern).escape('\\'));
    }
    if let Some(from) = query.from {
        statement = statement.filter(assets::taken_at.ge(from));
        total_query = total_query.filter(assets::taken_at.ge(from));
    }
    if let Some(to) = query.to {
        statement = statement.filter(assets::taken_at.le(to));
        total_query = total_query.filter(assets::taken_at.le(to));
    }
    if let Some(favorite) = query.favorite {
        statement = statement.filter(assets::favorite.eq(favorite));
        total_query = total_query.filter(assets::favorite.eq(favorite));
    }
    if let Some(media_type) = query.media_type {
        statement = statement.filter(assets::media_type.eq(media_type_to_str(media_type)));
        total_query = total_query.filter(assets::media_type.eq(media_type_to_str(media_type)));
    }
    let total = total_query
        .get_result::<i64>(&mut conn)
        .await
        .map_err(db_err)?;
    if let Some(cursor) = &query.cursor {
        statement = statement.filter(
            assets::taken_at.lt(cursor.taken_at).or(assets::taken_at
                .eq(cursor.taken_at)
                .and(assets::id.lt(cursor.id.clone()))),
        );
    }
    let rows = statement
        .order((assets::taken_at.desc(), assets::id.desc()))
        .limit(limit + 1)
        .load::<AssetRow>(&mut conn)
        .await
        .map_err(db_err)?;
    Ok(page_from_rows(rows, limit, total))
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
