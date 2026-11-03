//! Catalog indexing of the remote library.

use super::*;

impl AppCore {
    pub fn index_status(&self) -> IndexStatus {
        self.index.lock().unwrap().clone()
    }

    pub async fn start_index(&self, params: StartIndexParams) -> Result<IndexStatus, AppError> {
        if self.index_running.load(Ordering::SeqCst) {
            return Ok(self.index_status());
        }
        if self.catalog.library_root().await?.is_none() {
            return Err(AppError::internal("请先在资料库中选择图库根目录"));
        }
        self.index_cancel.store(false, Ordering::SeqCst);
        if params.full {
            self.catalog
                .set_setting("index_full_requested", "1")
                .await?;
        }
        {
            let mut status = self.index.lock().unwrap();
            status.phase = IndexPhase::Scanning;
            status.folders_done = 0;
            status.folders_total = 0;
            status.assets_seen = 0;
            status.started_at = Some(now_ms());
            status.error = None;
        }
        Ok(self.index_status())
    }

    pub fn index_mark_started(&self) {
        self.index_running.store(true, Ordering::SeqCst);
    }

    /// Ask a running pass to stop after the folder it is currently on.
    ///
    /// The pass stays resumable: folders already marked `done` are skipped by
    /// the next incremental run, and the phase becomes [`IndexPhase::Paused`]
    /// so the UI can offer "continue" instead of "start".
    pub fn cancel_index(&self) -> IndexStatus {
        if self.index_running.load(Ordering::SeqCst) {
            self.index_cancel.store(true, Ordering::SeqCst);
            let mut status = self.index.lock().unwrap();
            status.phase = IndexPhase::Paused;
            status.error = None;
        }
        self.index_status()
    }

    pub async fn run_index(&self, full: bool) {
        if self.index_running.swap(true, Ordering::SeqCst) {
            return;
        }
        self.index_cancel.store(false, Ordering::SeqCst);
        let result = self.run_index_inner(full).await;
        {
            let mut status = self.index.lock().unwrap();
            match result {
                Ok(()) => {
                    status.phase = if self.index_cancel.load(Ordering::SeqCst) {
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
        self.index_running.store(false, Ordering::SeqCst);
        let _ = self.catalog.set_setting("index_full_requested", "0").await;
    }

    pub(crate) async fn run_index_inner(&self, full: bool) -> Result<(), AppError> {
        let root = self
            .catalog
            .library_root()
            .await?
            .ok_or_else(|| AppError::internal("未选择图库根"))?;
        let root_info = self.cloud.folder_info(&root.uri).await?;
        let root = FolderRef {
            id: root_info.id.clone().into(),
            uri: root_info.uri.clone(),
            name: root_info.name.clone(),
            path: root_info.path.clone(),
        };
        self.upsert_remote_folder(&root, &root_info, None).await?;
        // The remote may hand out a different id for the same folder than the
        // one stored at login; keep the root pointer in step with the catalog.
        self.remember_library_root(&root).await?;

        let mut stack = vec![root];
        let mut seen = HashSet::new();
        // Only a pass that walked the whole tree knows which folders no longer
        // exist: a cancelled pass, and one that failed to list a folder, still
        // has subtrees it never reached.
        let mut complete = true;
        while let Some(folder) = stack.pop() {
            if self.index_cancel.load(Ordering::SeqCst) {
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
                let mut status = self.index.lock().unwrap();
                status.folders_done += 1;
                continue;
            }

            {
                let mut status = self.index.lock().unwrap();
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
                    let mut status = self.index.lock().unwrap();
                    status.folders_done += 1;
                    status.error = Some(err);
                    continue;
                }
            };

            let mut assets = Vec::new();
            for entry in &entries {
                if entry.is_folder {
                    let child = folder_ref(entry);
                    self.upsert_remote_folder(&child, entry, Some(&folder.id.0))
                        .await?;
                    {
                        let mut status = self.index.lock().unwrap();
                        status.folders_total += 1;
                    }
                    stack.push(child);
                } else {
                    let is_image = crate::mapping::media_type(None, &entry.name)
                        == crate::dto::MediaType::Image;
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
                let mut status = self.index.lock().unwrap();
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

    pub(crate) async fn list_all_entries(&self, uri: &str) -> Result<Vec<RemoteEntry>, AppError> {
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

    pub(crate) async fn upsert_remote_folder(
        &self,
        folder: &FolderRef,
        info: &RemoteEntry,
        parent_id: Option<&str>,
    ) -> Result<(), AppError> {
        self.catalog
            .upsert_folder(&FolderRecord {
                id: folder.id.0.clone(),
                parent_id: parent_id.map(str::to_owned),
                name: folder.name.clone(),
                uri: folder.uri.clone(),
                path: folder.path.clone(),
                updated_at: info.updated_at,
                is_album: false,
                cover_asset_id: info.metadata.get(COVER_KEY).cloned(),
                description: info.metadata.get(DESCRIPTION_KEY).cloned(),
            })
            .await
    }
}
