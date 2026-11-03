//! Assets, album flags and timeline/search queries.

use super::*;
use crate::query::*;
use crate::rows::*;
use crate::schema::assets;
use crate::schema::folders;
use crate::values::*;

pub(crate) async fn upsert_asset(cat: &SqliteCatalog, asset: &AssetRecord) -> Result<(), AppError> {
    let mut conn = cat.conn().await?;
    upsert_asset_row(&mut conn, asset).await
}

pub(crate) async fn mark_album(
    cat: &SqliteCatalog,
    folder_id: &str,
    is_album: bool,
) -> Result<(), AppError> {
    let mut conn = cat.conn().await?;
    diesel::update(folders::table.filter(folders::id.eq(folder_id)))
        .set(folders::is_album.eq(is_album))
        .execute(&mut conn)
        .await
        .map_err(db_err)?;
    Ok(())
}

/// Recompute `is_album` from the assets table and drop covers that no
/// longer exist.
///
/// Both steps read the small side (albums, covers) before writing, which
/// keeps the statements plain DSL instead of correlated subqueries.
pub(crate) async fn rebuild_albums(cat: &SqliteCatalog) -> Result<(), AppError> {
    let mut conn = cat.conn().await?;

    let album_ids = assets::table
        .filter(assets::media_type.eq_any(MEDIA_TYPES))
        .select(assets::folder_id)
        .distinct()
        .load::<String>(&mut conn)
        .await
        .map_err(db_err)?;

    diesel::update(folders::table)
        .set(folders::is_album.eq(false))
        .execute(&mut conn)
        .await
        .map_err(db_err)?;
    if !album_ids.is_empty() {
        diesel::update(folders::table.filter(folders::id.eq_any(&album_ids)))
            .set(folders::is_album.eq(true))
            .execute(&mut conn)
            .await
            .map_err(db_err)?;
    }

    let covers = folders::table
        .filter(folders::cover_asset_id.is_not_null())
        .select(folders::cover_asset_id)
        .load::<Option<String>>(&mut conn)
        .await
        .map_err(db_err)?;
    let covers: Vec<String> = covers.into_iter().flatten().collect();
    if !covers.is_empty() {
        let known = assets::table
            .filter(assets::id.eq_any(&covers))
            .select(assets::id)
            .load::<String>(&mut conn)
            .await
            .map_err(db_err)?;
        let stale: Vec<&String> = covers.iter().filter(|id| !known.contains(id)).collect();
        if !stale.is_empty() {
            diesel::update(folders::table.filter(folders::cover_asset_id.eq_any(stale)))
                .set(folders::cover_asset_id.eq(None::<String>))
                .execute(&mut conn)
                .await
                .map_err(db_err)?;
        }
    }
    Ok(())
}

pub(crate) async fn timeline_page(
    cat: &SqliteCatalog,
    cursor: Option<&TimelineCursor>,
    limit: u32,
) -> Result<TimelinePage, AppError> {
    let mut conn = cat.conn().await?;
    timeline(&mut conn, None, cursor, limit).await
}

/// Album list with counts and covers.
///
/// Three plain queries (albums, counts, newest media per album) beat one
/// correlated query: SQLite scans each table once and the assembly happens
/// in Rust.
pub(crate) async fn asset(cat: &SqliteCatalog, id: &str) -> Result<Option<AssetRecord>, AppError> {
    let mut conn = cat.conn().await?;
    let row = assets::table
        .filter(assets::id.eq(id))
        .select((
            AssetRow::as_select(),
            assets::uri,
            assets::exif_json,
            assets::remote_updated_at,
        ))
        .first::<(AssetRow, String, Option<String>, i64)>(&mut conn)
        .await
        .optional()
        .map_err(db_err)?;
    Ok(row.map(|(row, uri, exif_json, remote_updated_at)| {
        let summary = AssetSummary::from(row);
        AssetRecord {
            taken_at_source: summary.taken_at_source,
            summary,
            uri,
            exif_json,
            remote_updated_at,
        }
    }))
}

pub(crate) async fn set_favorite(
    cat: &SqliteCatalog,
    asset_id: &str,
    value: bool,
) -> Result<(), AppError> {
    let mut conn = cat.conn().await?;
    diesel::update(assets::table.filter(assets::id.eq(asset_id)))
        .set(assets::favorite.eq(value))
        .execute(&mut conn)
        .await
        .map_err(db_err)?;
    Ok(())
}

pub(crate) async fn move_asset(
    cat: &SqliteCatalog,
    asset_id: &str,
    folder_id: &str,
    uri: &str,
) -> Result<(), AppError> {
    let mut conn = cat.conn().await?;
    diesel::update(assets::table.filter(assets::id.eq(asset_id)))
        .set((assets::folder_id.eq(folder_id), assets::uri.eq(uri)))
        .execute(&mut conn)
        .await
        .map_err(db_err)?;
    Ok(())
}

pub(crate) async fn set_thumb_hash(
    cat: &SqliteCatalog,
    asset_id: &str,
    value: &str,
) -> Result<(), AppError> {
    let mut conn = cat.conn().await?;
    diesel::update(assets::table.filter(assets::id.eq(asset_id)))
        .set(assets::thumb_hash.eq(value))
        .execute(&mut conn)
        .await
        .map_err(db_err)?;
    Ok(())
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
