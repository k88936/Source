//! Queries shared by several port methods.

use super::asset::rows::AssetRow;
use super::asset::schema::assets;
use super::asset::values::*;
use super::*;

// ---------------------------------------------------------------------------
// Shared queries

/// Insert or update one asset row.
///
/// The update arm deliberately leaves `created_at`, `exif_parsed` and
/// `thumb_hash` alone: favorites survive a re-index, and the blur-up mosaic is
/// derived from the thumbnail, not from the remote metadata.
pub(crate) async fn upsert_asset_row(conn: &mut Conn, asset: &AssetRecord) -> Result<(), AppError> {
    let summary = &asset.summary;
    diesel::insert_into(assets::table)
        .values((
            assets::id.eq(summary.id.0.clone()),
            assets::folder_id.eq(summary.folder_id.0.clone()),
            assets::uri.eq(asset.uri.clone()),
            assets::name.eq(summary.name.clone()),
            assets::size.eq(summary.size),
            assets::updated_at.eq(asset.remote_updated_at),
            assets::remote_updated_at.eq(asset.remote_updated_at),
            assets::taken_at.eq(summary.taken_at),
            assets::taken_at_source.eq(taken_source_to_str(summary.taken_at_source)),
            assets::media_type.eq(media_type_to_str(summary.media_type)),
            assets::mime.eq(summary.mime.clone()),
            assets::width.eq(summary.width),
            assets::height.eq(summary.height),
            assets::duration_ms.eq(summary.duration_ms),
            assets::favorite.eq(summary.favorite),
            assets::exif_json.eq(asset.exif_json.clone()),
            assets::exif_parsed.eq(true),
        ))
        .on_conflict(assets::id)
        .do_update()
        .set((
            assets::folder_id.eq(excluded(assets::folder_id)),
            assets::uri.eq(excluded(assets::uri)),
            assets::name.eq(excluded(assets::name)),
            assets::size.eq(excluded(assets::size)),
            assets::updated_at.eq(excluded(assets::updated_at)),
            assets::remote_updated_at.eq(excluded(assets::remote_updated_at)),
            assets::taken_at.eq(excluded(assets::taken_at)),
            assets::taken_at_source.eq(excluded(assets::taken_at_source)),
            assets::media_type.eq(excluded(assets::media_type)),
            assets::mime.eq(excluded(assets::mime)),
            assets::width.eq(excluded(assets::width)),
            assets::height.eq(excluded(assets::height)),
            assets::duration_ms.eq(excluded(assets::duration_ms)),
            assets::favorite.eq(excluded(assets::favorite)),
            assets::exif_json.eq(excluded(assets::exif_json)),
            assets::exif_parsed.eq(true),
        ))
        .execute(conn)
        .await
        .map_err(db_err)?;
    Ok(())
}

/// One page of timeline assets, newest first.
pub(crate) async fn timeline(
    conn: &mut Conn,
    folder: Option<&str>,
    cursor: Option<&TimelineCursor>,
    limit: u32,
) -> Result<TimelinePage, AppError> {
    let limit = i64::from(limit.clamp(1, 200));
    let mut query = assets::table
        .filter(assets::media_type.eq_any(MEDIA_TYPES))
        .select(AssetRow::as_select())
        .into_boxed::<Sqlite>();
    if let Some(folder) = folder {
        query = query.filter(assets::folder_id.eq(folder));
    }
    if let Some(cursor) = cursor {
        query = query.filter(
            assets::taken_at.lt(cursor.taken_at).or(assets::taken_at
                .eq(cursor.taken_at)
                .and(assets::id.lt(cursor.id.clone()))),
        );
    }
    let rows = query
        .order((assets::taken_at.desc(), assets::id.desc()))
        .limit(limit + 1)
        .load::<AssetRow>(conn)
        .await
        .map_err(db_err)?;
    Ok(page_from_rows(rows, limit))
}

pub(crate) fn page_from_rows(rows: Vec<AssetRow>, limit: i64) -> TimelinePage {
    let mut items: Vec<AssetSummary> = rows.into_iter().map(AssetSummary::from).collect();
    let mut next = None;
    if items.len() as i64 > limit {
        items.truncate(limit as usize);
        if let Some(last) = items.last() {
            next = Some(TimelineCursor {
                taken_at: last.taken_at,
                id: last.id.0.clone(),
            });
        }
    }
    TimelinePage { items, next }
}
