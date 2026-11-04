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
    async fn timeline_page(
        &self,
        cursor: Option<&TimelineCursor>,
        limit: u32,
    ) -> Result<TimelinePage, AppError>;
    async fn search(&self, query: &SearchQuery) -> Result<TimelinePage, AppError>;
}

pub(crate) async fn timeline_page(
    cat: &SqliteCatalog,
    cursor: Option<&TimelineCursor>,
    limit: u32,
) -> Result<TimelinePage, AppError> {
    let mut conn = cat.conn().await?;
    timeline(&mut conn, None, cursor, limit).await
}

pub(crate) async fn search(
    cat: &SqliteCatalog,
    query: &SearchQuery,
) -> Result<TimelinePage, AppError> {
    let mut conn = cat.conn().await?;
    let limit = i64::from(query.limit.clamp(1, 200));
    let mut statement = assets::table
        .filter(assets::media_type.eq_any(MEDIA_TYPES))
        .select(AssetRow::as_select())
        .into_boxed::<Sqlite>();
    let needle = query.query.trim();
    if !needle.is_empty() {
        let escaped = needle
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_");
        statement = statement.filter(assets::name.like(format!("%{escaped}%")).escape('\\'));
    }
    if let Some(from) = query.from {
        statement = statement.filter(assets::taken_at.ge(from));
    }
    if let Some(to) = query.to {
        statement = statement.filter(assets::taken_at.le(to));
    }
    if let Some(favorite) = query.favorite {
        statement = statement.filter(assets::favorite.eq(favorite));
    }
    if let Some(media_type) = query.media_type {
        statement = statement.filter(assets::media_type.eq(media_type_to_str(media_type)));
    }
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
    Ok(page_from_rows(rows, limit))
}

#[async_trait]
impl TimelineStore for SqliteCatalog {
    async fn timeline_page(
        &self,
        cursor: Option<&TimelineCursor>,
        limit: u32,
    ) -> Result<TimelinePage, AppError> {
        timeline_page(self, cursor, limit).await
    }

    async fn search(&self, query: &SearchQuery) -> Result<TimelinePage, AppError> {
        search(self, query).await
    }
}
