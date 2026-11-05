//! Albums: the remote folders a user added, and the local albums syncing into
//! them.
//!
//! An album is a folder, not a tree. Its assets are the folder's direct media
//! files, so refreshing an album is one listing one level deep: no recursion, no
//! sub-folders and no non-media entries.

use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;
use ts_rs::TS;

use crate::error::{AppError, ErrorCode};
use crate::mapping::{COVER_KEY, DESCRIPTION_KEY, asset_record, entry_media_type};
use crate::media::{
    LocalAsset, LocalAssetPage, LocalAssetPageParams, LocalBrowsePage, LocalMediaPort,
};
use crate::remote::RemoteStorage;
use crate::store::{
    AlbumId, AlbumStore, AlbumSummary, AssetId, AssetRecord, AssetStore, AssetSummary, FolderId,
    FolderRecord, FolderStore, LocalAlbumRecord, LocalStore, MediaType, Run, RunKey, RunStore,
    SYNC_DONE, SYNC_FAILED, TimelineCursor, TimelinePage, TimelineStore,
};
use crate::task::{Retry, Runner};
use crate::time::now_ms;

use super::run::{RunSink, RunTracker};
use super::{clamp_limit, folder_record, folder_ref};

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AddAlbumParams {
    pub uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AlbumPageParams {
    pub album_id: AlbumId,
    pub cursor: Option<TimelineCursor>,
    pub limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AlbumParams {
    pub album_id: AlbumId,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SetAlbumCoverParams {
    pub album_id: AlbumId,
    pub asset_id: AssetId,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SetAlbumDescriptionParams {
    pub album_id: AlbumId,
    pub description: String,
}

/// Move a batch of assets into an album (a folder move on Cloudreve).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MoveAssetsParams {
    pub asset_ids: Vec<AssetId>,
    pub album_id: AlbumId,
}

/// A registered local album together with where it syncs to.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LocalAlbumSummary {
    pub id: String,
    pub uri: String,
    pub name: String,
    pub album_id: AlbumId,
    pub album_name: String,
    /// Direct media files, counted on the device every time it is asked.
    pub asset_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AddLocalAlbumParams {
    /// The directory as the media adapter names it.
    pub uri: String,
    /// What the picker called it; kept so a bucket rename does not blank the row.
    pub name: String,
    pub album_id: AlbumId,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LocalAlbumParams {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LocalBrowseParams {
    /// The directory to list; `None` starts where the platform browses from.
    pub parent: Option<String>,
}

pub struct AlbumsService<R, C, M> {
    cloud: R,
    catalog: C,
    local: M,
    /// The token of the index pass running now, so a stop can reach it.
    indexing: Mutex<Option<CancellationToken>>,
}

impl<R, C, M> AlbumsService<R, C, M> {
    pub fn new(cloud: R, catalog: C, local: M) -> Self {
        Self {
            cloud,
            catalog,
            local,
            indexing: Mutex::new(None),
        }
    }
}

/// How many albums an index pass lists at once.
///
/// Three spreads the round trips without hammering a phone uplink that uploads
/// are sharing at the same time.
const INDEX_CONCURRENCY: usize = 3;

/// Which pass this is. The store compares keys and never learns what one means.
const INDEX_RUN: RunKey = RunKey::new("index");

impl<R, C, M> AlbumsService<R, C, M>
where
    R: RemoteStorage + 'static,
    C: AlbumStore
        + AssetStore
        + FolderStore
        + TimelineStore
        + LocalStore
        + RunStore
        + Clone
        + 'static,
    M: LocalMediaPort + 'static,
{
    pub async fn list_albums(&self) -> Result<Vec<AlbumSummary>, AppError> {
        self.catalog.list_albums().await
    }

    pub async fn get_album(&self, album_id: &str) -> Result<Option<AlbumSummary>, AppError> {
        self.catalog.get_album(album_id).await
    }

    pub async fn get_album_page(&self, params: AlbumPageParams) -> Result<TimelinePage, AppError> {
        self.catalog
            .get_album_page(
                &params.album_id.0,
                params.cursor.as_ref(),
                clamp_limit(params.limit),
            )
            .await
    }

    /// Add a folder as an album and list it once, so it is never shown empty
    /// while the server has media in it.
    pub async fn add_album(&self, params: AddAlbumParams) -> Result<AlbumSummary, AppError> {
        let info = self.cloud.get_folder_info(&params.uri).await?;
        if !info.is_folder {
            return Err(AppError::internal("只能把文件夹添加为相册"));
        }
        let folder = folder_ref(&info);
        self.catalog
            .save_album(&folder_record(&folder, &info))
            .await?;
        self.refresh_album(&folder.id.0).await
    }

    /// Forget an album. The folder on the server is left alone, and every local
    /// album that was syncing into it goes with the row.
    pub async fn remove_album(&self, album_id: &str) -> Result<(), AppError> {
        self.catalog.delete_album(album_id).await
    }

    /// List one album's direct media and make the catalog match.
    ///
    /// A folder that was deleted or renamed on the server keeps its row and its
    /// assets: the failure is recorded on the album so the UI can offer to point
    /// it somewhere else, instead of the album silently disappearing.
    pub async fn refresh_album(&self, album_id: &str) -> Result<AlbumSummary, AppError> {
        self.refresh(album_id, &CancellationToken::new()).await
    }

    /// The newest index pass, whatever state it is in.
    pub async fn get_index_run(&self) -> Result<Option<Run>, AppError> {
        self.catalog.get_run(INDEX_RUN).await
    }

    /// Start listing every album, and return the pass instead of waiting for it.
    ///
    /// A pass already running is handed back as it is, so a double trigger never
    /// starts two. Every album ends on its own row, and the pass ends with a
    /// count - which is what the albums page shows while it works.
    pub async fn start_index(self: &Arc<Self>, sink: RunSink) -> Result<Run, AppError> {
        let ids = self
            .catalog
            .list_albums()
            .await?
            .into_iter()
            .map(|album| album.id.0)
            .collect::<Vec<_>>();
        let (run, started) = self.catalog.begin_run(INDEX_RUN, ids.len() as u32).await?;
        if !started {
            return Ok(run);
        }
        let cancel = CancellationToken::new();
        *self.indexing.lock().unwrap() = Some(cancel.clone());
        let tracker = RunTracker::new(self.catalog.clone(), &run, sink, cancel);
        tracker.announce(&run);

        let albums = Arc::clone(self);
        tokio::spawn(async move {
            let runner = Runner::new(INDEX_CONCURRENCY, Retry::default());
            let batch = Arc::clone(&albums);
            runner
                .run(ids, tracker.get_cancel(), |id, _| {
                    let albums = Arc::clone(&batch);
                    let tracker = tracker.clone();
                    async move { albums.index_album(&id, &tracker).await }
                })
                .await;
            // The pass is over once every attempt has returned, whatever the
            // stop button did meanwhile.
            let _ = tracker.close().await;
            *albums.indexing.lock().unwrap() = None;
        });
        Ok(run)
    }

    /// Stop the index pass after the albums already in flight.
    pub fn stop_index(&self) {
        if let Some(cancel) = self.indexing.lock().unwrap().as_ref() {
            cancel.cancel();
        }
    }

    /// Refresh one album, stopping between listing pages when `cancel` fires.
    async fn refresh(
        &self,
        album_id: &str,
        cancel: &CancellationToken,
    ) -> Result<AlbumSummary, AppError> {
        let album = self
            .catalog
            .get_folder(album_id)
            .await?
            .ok_or_else(|| AppError::not_found("相册不存在"))?;
        if let Err(err) = self.sync_album(&album, cancel).await {
            // A stop is not a failure of the album: leave its last state alone.
            if err.code != ErrorCode::Cancelled {
                self.catalog
                    .mark_folder(&album.id, SYNC_FAILED, Some(&err.message), None, now_ms())
                    .await?;
            }
            return Err(err);
        }
        self.catalog
            .get_album(album_id)
            .await?
            .ok_or_else(|| AppError::not_found("相册不存在"))
    }

    /// One entry of an index pass.
    async fn index_album(&self, album_id: &str, tracker: &RunTracker<C>) -> Result<(), AppError> {
        match self.refresh(album_id, tracker.get_cancel()).await {
            Ok(_) => tracker.step(true).await,
            Err(err) if err.code == ErrorCode::Cancelled => Ok(()),
            Err(_) => tracker.step(false).await,
        }
    }

    async fn sync_album(
        &self,
        album: &FolderRecord,
        cancel: &CancellationToken,
    ) -> Result<(), AppError> {
        let folder_id = FolderId::new(album.id.clone());
        // Every id the listing returned, for the prune below. The rows
        // themselves go in page by page, so a big album never sits in memory
        // whole and a page boundary is a place the pass can be stopped.
        let mut listed = Vec::new();
        let mut token: Option<String> = None;
        let mut remote_updated_at = 0;
        loop {
            if cancel.is_cancelled() {
                return Err(AppError::cancelled());
            }
            let page = self
                .cloud
                .list_entries(&album.uri, token.as_deref())
                .await?;
            let records: Vec<AssetRecord> = page
                .entries
                .iter()
                .filter(|entry| is_media_entry(entry))
                .map(|entry| {
                    remote_updated_at = remote_updated_at.max(entry.updated_at);
                    asset_record(entry, &folder_id, None)
                })
                .collect();
            listed.extend(records.iter().map(|record| record.summary.id.0.clone()));
            self.catalog.upsert_album_assets(&records).await?;
            token = page.next;
            if token.is_none() {
                break;
            }
        }
        // Only a listing that ran to the end can tell a removed file from a
        // page this pass has not reached.
        self.catalog.prune_album_assets(&album.id, &listed).await?;
        self.catalog
            .mark_folder(
                &album.id,
                SYNC_DONE,
                None,
                Some(remote_updated_at),
                now_ms(),
            )
            .await?;
        Ok(())
    }

    pub async fn set_album_cover(
        &self,
        album_id: &str,
        asset_id: &str,
    ) -> Result<AlbumSummary, AppError> {
        let folder = self
            .catalog
            .get_folder(album_id)
            .await?
            .ok_or_else(|| AppError::not_found("相册不存在"))?;
        let patch = (COVER_KEY.to_string(), Some(asset_id.to_string()));
        self.cloud.patch_metadata(&folder.uri, &[patch]).await?;
        self.catalog
            .set_album_cover(album_id, Some(asset_id))
            .await?;
        self.catalog
            .get_album(album_id)
            .await?
            .ok_or_else(|| AppError::not_found("相册不存在"))
    }

    pub async fn set_album_description(
        &self,
        album_id: &str,
        description: &str,
    ) -> Result<AlbumSummary, AppError> {
        let folder = self
            .catalog
            .get_folder(album_id)
            .await?
            .ok_or_else(|| AppError::not_found("相册不存在"))?;
        let value = if description.trim().is_empty() {
            None
        } else {
            Some(description.to_string())
        };
        self.cloud
            .patch_metadata(&folder.uri, &[(DESCRIPTION_KEY.to_string(), value.clone())])
            .await?;
        self.catalog
            .set_album_description(album_id, value.as_deref())
            .await?;
        self.catalog
            .get_album(album_id)
            .await?
            .ok_or_else(|| AppError::not_found("相册不存在"))
    }

    /// Move assets into an album.
    ///
    /// In Cloudreve an album is a folder, so this is a real move: the files
    /// leave their current folder. The catalog rows are re-pointed only after
    /// the remote move succeeded.
    pub async fn move_assets_to_album(
        &self,
        asset_ids: &[String],
        album_id: &str,
    ) -> Result<Vec<AssetSummary>, AppError> {
        if asset_ids.is_empty() {
            return Ok(Vec::new());
        }
        let folder = self
            .catalog
            .get_folder(album_id)
            .await?
            .ok_or_else(|| AppError::not_found("相册不存在"))?;
        let mut records = Vec::with_capacity(asset_ids.len());
        for id in asset_ids {
            let record = self
                .catalog
                .get_asset(id)
                .await?
                .ok_or_else(|| AppError::not_found("资产不存在"))?;
            records.push(record);
        }
        let uris: Vec<String> = records.iter().map(|r| r.uri.clone()).collect();
        let new_uris = self.cloud.move_assets(&uris, &folder.uri).await?;
        if new_uris.len() != records.len() {
            return Err(AppError::internal("移动结果与请求不一致"));
        }

        let album_id = FolderId::new(folder.id);
        let mut moved = Vec::with_capacity(records.len());
        for (record, new_uri) in records.into_iter().zip(new_uris) {
            let asset_id = record.summary.id.0.clone();
            self.catalog
                .move_asset(&asset_id, &album_id.0, &new_uri)
                .await?;
            moved.push(AssetSummary {
                folder_id: album_id.clone(),
                ..record.summary
            });
        }
        Ok(moved)
    }

    // ---------------------------------------------------------------------
    // Local albums
    // ---------------------------------------------------------------------

    pub async fn local_album_list(&self) -> Result<Vec<LocalAlbumSummary>, AppError> {
        let albums = self.catalog.list_local_albums().await?;
        let mut summaries = Vec::with_capacity(albums.len());
        for album in albums {
            summaries.push(self.local_album_to_summary(album).await?);
        }
        Ok(summaries)
    }

    /// One step of the device library browser the picker is built on.
    pub async fn browse_local(&self, parent: Option<&str>) -> Result<LocalBrowsePage, AppError> {
        self.local.browse(parent).await
    }

    /// The direct media files of one local album, newest first.
    pub async fn get_local_asset_page(
        &self,
        params: LocalAssetPageParams,
    ) -> Result<LocalAssetPage, AppError> {
        self.local
            .list_assets(
                &params.album_uri,
                params.cursor.as_deref(),
                clamp_limit(params.limit),
            )
            .await
    }

    pub async fn get_local_asset(&self, id: &str) -> Result<Option<LocalAsset>, AppError> {
        self.local.get_asset(id).await
    }

    /// Register a device album - a directory, or a phone library bucket - as
    /// the source of one album.
    ///
    /// A local album only ever exists to sync somewhere, so the target album is
    /// part of the same write as the registration.
    pub async fn add_local_album(
        &self,
        params: AddLocalAlbumParams,
    ) -> Result<LocalAlbumSummary, AppError> {
        let album = self
            .catalog
            .get_album(&params.album_id.0)
            .await?
            .ok_or_else(|| AppError::not_found("相册不存在"))?;
        self.catalog
            .add_local_album(&params.uri, &params.name, &album.id.0)
            .await?;
        let record = self
            .catalog
            .get_local_album(&params.uri)
            .await?
            .ok_or_else(|| AppError::not_found("本机相册不存在"))?;
        self.local_album_to_summary(record).await
    }

    /// Stop syncing a device directory. Files already uploaded stay where they
    /// are, and the album is untouched.
    pub async fn remove_local_album(&self, id: &str) -> Result<(), AppError> {
        self.catalog.remove_local_album(id).await
    }

    async fn local_album_to_summary(
        &self,
        album: LocalAlbumRecord,
    ) -> Result<LocalAlbumSummary, AppError> {
        let asset_count = self.local.count_assets(&album.uri).await?;
        Ok(LocalAlbumSummary {
            id: album.id,
            uri: album.uri,
            name: album.name,
            album_id: AlbumId(album.album_id),
            album_name: album.album_name,
            asset_count,
        })
    }
}

fn is_media_entry(entry: &crate::remote::RemoteEntry) -> bool {
    !entry.is_folder && matches!(entry_media_type(entry), MediaType::Image | MediaType::Video)
}
