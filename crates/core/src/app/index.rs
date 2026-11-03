//! Catalog indexing of the remote library.

use std::collections::HashSet;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::AppError;
use crate::mapping::asset_record;
use crate::remote::{RemoteEntry, RemoteStorage};
use crate::store::{AccountStore, AlbumStore, FolderRef, FolderStore, MediaType, SettingsStore};

use super::library_root;
use crate::time::now_ms;

use super::{folder_ref, is_media, upsert_remote_folder};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum IndexPhase {
    Idle,
    Scanning,
    Paused,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct IndexStatus {
    pub phase: IndexPhase,
    pub folders_done: u32,
    pub folders_total: u32,
    pub assets_seen: i64,
    pub started_at: Option<i64>,
    pub error: Option<AppError>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct StartIndexParams {
    #[serde(default)]
    pub full: bool,
}

pub struct IndexService<R, C> {
    cloud: R,
    catalog: C,
    status: Mutex<IndexStatus>,
    running: AtomicBool,
    cancel: AtomicBool,
}

impl<R, C> IndexService<R, C> {
    pub fn new(cloud: R, catalog: C) -> Self {
        Self {
            cloud,
            catalog,
            status: Mutex::new(IndexStatus {
                phase: IndexPhase::Idle,
                folders_done: 0,
                folders_total: 0,
                assets_seen: 0,
                started_at: None,
                error: None,
            }),
            running: AtomicBool::new(false),
            cancel: AtomicBool::new(false),
        }
    }
}

impl<R, C> IndexService<R, C>
where
    R: RemoteStorage,
    C: AlbumStore + FolderStore + AccountStore + SettingsStore,
{
    pub fn status(&self) -> IndexStatus {
        self.status.lock().unwrap().clone()
    }

    pub async fn start(&self, params: StartIndexParams) -> Result<IndexStatus, AppError> {
        if self.running.load(Ordering::SeqCst) {
            return Ok(self.status());
        }
        if library_root(&self.catalog).await?.is_none() {
            return Err(AppError::internal("请先在资料库中选择图库根目录"));
        }
        self.cancel.store(false, Ordering::SeqCst);
        if params.full {
            self.catalog
                .set_setting("index_full_requested", "1")
                .await?;
        }
        {
            let mut status = self.status.lock().unwrap();
            status.phase = IndexPhase::Scanning;
            status.folders_done = 0;
            status.folders_total = 0;
            status.assets_seen = 0;
            status.started_at = Some(now_ms());
            status.error = None;
        }
        Ok(self.status())
    }

    pub fn mark_started(&self) {
        self.running.store(true, Ordering::SeqCst);
    }

    /// Ask a running pass to stop after the folder it is currently on.
    ///
    /// The pass stays resumable: folders already marked `done` are skipped by
    /// the next incremental run, and the phase becomes [`IndexPhase::Paused`]
    /// so the UI can offer "continue" instead of "start".
    pub fn cancel_pass(&self) -> IndexStatus {
        if self.running.load(Ordering::SeqCst) {
            self.cancel.store(true, Ordering::SeqCst);
            let mut status = self.status.lock().unwrap();
            status.phase = IndexPhase::Paused;
            status.error = None;
        }
        self.status()
    }

    pub async fn run(&self, full: bool) {
        if self.running.swap(true, Ordering::SeqCst) {
            return;
        }
        self.cancel.store(false, Ordering::SeqCst);
        let result = self.run_inner(full).await;
        {
            let mut status = self.status.lock().unwrap();
            match result {
                Ok(()) => {
                    status.phase = if self.cancel.load(Ordering::SeqCst) {
                        IndexPhase::Paused
                    } else {
                        IndexPhase::Idle
                    };
                    status.error = None;
                }
                Err(err) => {
                    status.phase = IndexPhase::Failed;
                    status.error = Some(err);
                }
            }
        }
        self.running.store(false, Ordering::SeqCst);
        let _ = self.catalog.set_setting("index_full_requested", "0").await;
    }

    async fn run_inner(&self, full: bool) -> Result<(), AppError> {
        let account = self
            .catalog
            .current_account()
            .await?
            .ok_or_else(|| AppError::internal("尚未选择服务器"))?;
        let root = account
            .library_root
            .ok_or_else(|| AppError::internal("未选择图库根"))?;
        let root_info = self.cloud.folder_info(&root.uri).await?;
        let root = FolderRef {
            id: root_info.id.clone().into(),
            uri: root_info.uri.clone(),
            name: root_info.name.clone(),
            path: root_info.path.clone(),
        };
        upsert_remote_folder(&self.catalog, &root, &root_info, None).await?;
        // The remote may hand out a different id for the same folder than the
        // one stored at login; keep the root pointer in step with the catalog.
        self.catalog
            .set_library_root(&account.base_url, &root)
            .await?;

        let mut stack = vec![root];
        let mut seen = HashSet::new();
        // Only a pass that walked the whole tree knows which folders no longer
        // exist: a cancelled pass, and one that failed to list a folder, still
        // has subtrees it never reached.
        let mut complete = true;
        while let Some(folder) = stack.pop() {
            if self.cancel.load(Ordering::SeqCst) {
                complete = false;
                break;
            }
            if !seen.insert(folder.id.0.clone()) {
                continue;
            }
            let info = self.cloud.folder_info(&folder.uri).await?;
            let state = self.catalog.folder_state(&folder.id.0).await?;
            if !full
                && state
                    .as_ref()
                    .map(|s| s.is_done() && s.remote_updated_at == Some(info.updated_at))
                    .unwrap_or(false)
            {
                let mut status = self.status.lock().unwrap();
                status.folders_done += 1;
                continue;
            }

            {
                let mut status = self.status.lock().unwrap();
                status.folders_total = status.folders_total.max(status.folders_done + 1);
            }

            let listed = self.list_all_entries(&folder.uri).await;
            let entries = match listed {
                Ok(v) => v,
                Err(err) => {
                    complete = false;
                    self.catalog
                        .mark_folder(&folder.id.0, "failed", Some(&err.message), None, now_ms())
                        .await?;
                    let mut status = self.status.lock().unwrap();
                    status.folders_done += 1;
                    status.error = Some(err);
                    continue;
                }
            };

            let mut assets = Vec::new();
            for entry in &entries {
                if entry.is_folder {
                    let child = folder_ref(entry);
                    upsert_remote_folder(&self.catalog, &child, entry, Some(&folder.id.0)).await?;
                    {
                        let mut status = self.status.lock().unwrap();
                        status.folders_total += 1;
                    }
                    stack.push(child);
                } else {
                    let is_image =
                        crate::mapping::media_type(None, &entry.name) == MediaType::Image;
                    let client_exif = if is_image && !entry.metadata.contains_key("exif:taken_at") {
                        self.cloud
                            .client_taken_at(&entry.uri)
                            .await
                            .ok()
                            .flatten()
                            .map(|ts| crate::exif::ExifData {
                                taken_at: Some(ts),
                                width: None,
                                height: None,
                            })
                    } else {
                        None
                    };
                    assets.push(asset_record(entry, &folder.id, client_exif.as_ref()));
                }
            }

            let count = assets.iter().filter(|a| is_media(&a.summary)).count() as i64;
            self.catalog
                .replace_folder_assets(&folder.id.0, &assets, now_ms())
                .await?;
            self.catalog.mark_album(&folder.id.0, count > 0).await?;
            self.catalog
                .mark_folder(&folder.id.0, "done", None, Some(info.updated_at), now_ms())
                .await?;
            {
                let mut status = self.status.lock().unwrap();
                status.folders_done += 1;
                status.assets_seen += count;
            }
        }
        if full && complete {
            self.catalog.retain_folders(&seen).await?;
        }
        self.catalog.rebuild_albums().await?;
        Ok(())
    }

    async fn list_all_entries(&self, uri: &str) -> Result<Vec<RemoteEntry>, AppError> {
        let mut out = Vec::new();
        let mut token: Option<String> = None;
        loop {
            let page = self.cloud.list_entries(uri, token.as_deref()).await?;
            out.extend(page.entries);
            match page.next {
                Some(next) if Some(&next) != token.as_ref() => token = Some(next),
                _ => break,
            }
        }
        Ok(out)
    }
}
