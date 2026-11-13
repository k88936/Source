//! Indexed assets and the per-asset flags the UI can toggle: the contract and
//! the diesel statements behind it.

use async_trait::async_trait;

use self::schema::assets;
use super::query::*;
use super::*;

mod dto;
pub(crate) mod rows;
pub(crate) mod schema;
pub(crate) mod values;

pub use dto::*;

use self::rows::*;

#[async_trait]
pub trait AssetStore: Send + Sync {
    async fn upsert_asset(&self, asset: &AssetRecord) -> Result<(), CatalogError>;
    /// Re-point an asset at another folder and URI after a remote move.
    async fn move_asset(
        &self,
        asset_id: &str,
        folder_id: &str,
        uri: &str,
    ) -> Result<(), CatalogError>;
    async fn get_asset(&self, id: &str) -> Result<Option<AssetRecord>, CatalogError>;
    async fn set_favorite(&self, asset_id: &str, value: bool) -> Result<(), CatalogError>;
    /// Store the blur-up mosaic for a tile. Written once per asset, lazily.
    async fn set_asset_thumb_hash(&self, asset_id: &str, value: &str) -> Result<(), CatalogError>;
    /// The assets a filter holds between two anchors, newest first.
    ///
    /// A subscription sends this once and then only the rows that crossed an
    /// anchor, so the interval is the whole bound: there is no size to agree on.
    async fn asset_window(
        &self,
        folder: Option<&str>,
        from: Option<TimelineCursor>,
        to: Option<TimelineCursor>,
    ) -> Result<Vec<AssetSummary>, CatalogError>;
    /// The key `depth` rows below `after`, so a page can cut its next interval
    /// without reading the rows in between.
    async fn asset_edge(
        &self,
        folder: Option<&str>,
        after: Option<TimelineCursor>,
        depth: u32,
    ) -> Result<Option<TimelineCursor>, CatalogError>;
    /// The summaries of these ids, in no particular order.
    async fn asset_summaries(&self, ids: &[&str]) -> Result<Vec<AssetSummary>, CatalogError>;
    /// How many assets the scope holds, so a page can say how long its list is
    /// before it has scrolled through it.
    async fn asset_count(&self, folder: Option<&str>) -> Result<u32, CatalogError>;
}

pub(crate) async fn upsert_asset(
    cat: &SqliteCatalog,
    asset: &AssetRecord,
) -> Result<(), CatalogError> {
    let mut conn = cat.conn().await?;
    upsert_asset_row(&mut conn, asset).await
}

pub(crate) async fn get_asset(
    cat: &SqliteCatalog,
    id: &str,
) -> Result<Option<AssetRecord>, CatalogError> {
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
) -> Result<(), CatalogError> {
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
) -> Result<(), CatalogError> {
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
) -> Result<(), CatalogError> {
    let mut conn = cat.conn().await?;
    diesel::update(assets::table.filter(assets::id.eq(asset_id)))
        .set(assets::thumb_hash.eq(value))
        .execute(&mut conn)
        .await
        .map_err(db_err)?;
    Ok(())
}

/// The assets a filter holds between two anchors, newest first.
///
/// `from` is the oldest row a window holds and `to` the newest one it stops
/// before, so the deepest row of one window is where the next one stops and
/// consecutive windows neither overlap nor leave a hole. An absent anchor is
/// that end of the timeline: `from: None` runs on to the oldest row, `to: None`
/// starts at the newest.
pub(crate) async fn asset_window(
    cat: &SqliteCatalog,
    folder: Option<&str>,
    from: Option<TimelineCursor>,
    to: Option<TimelineCursor>,
) -> Result<Vec<AssetSummary>, CatalogError> {
    let mut conn = cat.conn().await?;
    let mut query = assets::table
        .filter(assets::media_type.eq_any(MEDIA_TYPES))
        .select(AssetRow::as_select())
        .into_boxed::<Sqlite>();
    if let Some(folder) = folder {
        query = query.filter(assets::folder_id.eq(folder));
    }
    if let Some(to) = to {
        query = query.filter(
            assets::taken_at
                .lt(to.taken_at)
                .or(assets::taken_at.eq(to.taken_at).and(assets::id.lt(to.id))),
        );
    }
    if let Some(from) = from {
        query = query.filter(
            assets::taken_at.gt(from.taken_at).or(assets::taken_at
                .eq(from.taken_at)
                .and(assets::id.ge(from.id))),
        );
    }
    let rows = query
        .order((assets::taken_at.desc(), assets::id.desc()))
        .load::<AssetRow>(&mut conn)
        .await
        .map_err(db_err)?;
    Ok(rows.into_iter().map(AssetSummary::from).collect())
}

/// The key `depth` rows below `after`, or `None` when the scope ends first.
///
/// The offset counts from `after` in the indexed order, so a page can cut its
/// next interval without reading the rows in between, and scrolling deeper
/// costs the same as the first screen.
pub(crate) async fn asset_edge(
    cat: &SqliteCatalog,
    folder: Option<&str>,
    after: Option<TimelineCursor>,
    depth: u32,
) -> Result<Option<TimelineCursor>, CatalogError> {
    let mut conn = cat.conn().await?;
    let depth = i64::from(depth.max(1));
    let mut query = assets::table
        .filter(assets::media_type.eq_any(MEDIA_TYPES))
        .select((assets::taken_at, assets::id))
        .into_boxed::<Sqlite>();
    if let Some(folder) = folder {
        query = query.filter(assets::folder_id.eq(folder));
    }
    if let Some(after) = after {
        query = query.filter(
            assets::taken_at.lt(after.taken_at).or(assets::taken_at
                .eq(after.taken_at)
                .and(assets::id.lt(after.id))),
        );
    }
    let key = query
        .order((assets::taken_at.desc(), assets::id.desc()))
        .limit(1)
        .offset(depth - 1)
        .first::<(i64, String)>(&mut conn)
        .await
        .optional()
        .map_err(db_err)?;
    Ok(key.map(|(taken_at, id)| TimelineCursor { taken_at, id }))
}

/// The summaries of these ids. A commit names the rows it touched; this is how
/// a view reads exactly those back instead of the whole set around them.
pub(crate) async fn asset_summaries(
    cat: &SqliteCatalog,
    ids: &[&str],
) -> Result<Vec<AssetSummary>, CatalogError> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let mut conn = cat.conn().await?;
    let rows = assets::table
        .filter(assets::media_type.eq_any(MEDIA_TYPES))
        .filter(assets::id.eq_any(ids))
        .select(AssetRow::as_select())
        .load::<AssetRow>(&mut conn)
        .await
        .map_err(db_err)?;
    Ok(rows.into_iter().map(AssetSummary::from).collect())
}

pub(crate) async fn asset_count(
    cat: &SqliteCatalog,
    folder: Option<&str>,
) -> Result<u32, CatalogError> {
    let mut conn = cat.conn().await?;
    let mut query = assets::table
        .filter(assets::media_type.eq_any(MEDIA_TYPES))
        .select(diesel::dsl::count_star())
        .into_boxed::<Sqlite>();
    if let Some(folder) = folder {
        query = query.filter(assets::folder_id.eq(folder));
    }
    let count = query.get_result::<i64>(&mut conn).await.map_err(db_err)?;
    Ok(count.max(0) as u32)
}

#[async_trait]
impl AssetStore for SqliteCatalog {
    async fn upsert_asset(&self, asset_record: &AssetRecord) -> Result<(), CatalogError> {
        upsert_asset(self, asset_record).await
    }

    async fn move_asset(
        &self,
        asset_id: &str,
        folder_id: &str,
        uri: &str,
    ) -> Result<(), CatalogError> {
        move_asset(self, asset_id, folder_id, uri).await
    }

    async fn get_asset(&self, id: &str) -> Result<Option<AssetRecord>, CatalogError> {
        get_asset(self, id).await
    }

    async fn set_favorite(&self, asset_id: &str, value: bool) -> Result<(), CatalogError> {
        set_favorite(self, asset_id, value).await
    }

    async fn set_asset_thumb_hash(&self, asset_id: &str, value: &str) -> Result<(), CatalogError> {
        set_thumb_hash(self, asset_id, value).await
    }

    async fn asset_window(
        &self,
        folder: Option<&str>,
        from: Option<TimelineCursor>,
        to: Option<TimelineCursor>,
    ) -> Result<Vec<AssetSummary>, CatalogError> {
        asset_window(self, folder, from, to).await
    }

    async fn asset_edge(
        &self,
        folder: Option<&str>,
        after: Option<TimelineCursor>,
        depth: u32,
    ) -> Result<Option<TimelineCursor>, CatalogError> {
        asset_edge(self, folder, after, depth).await
    }

    async fn asset_summaries(&self, ids: &[&str]) -> Result<Vec<AssetSummary>, CatalogError> {
        asset_summaries(self, ids).await
    }

    async fn asset_count(&self, folder: Option<&str>) -> Result<u32, CatalogError> {
        asset_count(self, folder).await
    }
}
