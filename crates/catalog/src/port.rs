//! `CatalogPort` and `UploadSessionPort` wired to the diesel statements in the
//! sibling modules.
//!
//! Each trait method is a one-line delegation: the implementations live next to
//! the tables they touch, and this file keeps the trait impl single and obvious.

use std::collections::HashSet;

use super::*;

#[async_trait]
impl CatalogPort for SqliteCatalog {
    async fn save_server(&self, server_record: &StoredServer) -> Result<(), AppError> {
        crate::server::save_server(self, server_record).await
    }

    async fn load_server(&self) -> Result<Option<StoredServer>, AppError> {
        crate::server::load_server(self).await
    }

    async fn clear_server(&self) -> Result<(), AppError> {
        crate::server::clear_server(self).await
    }

    async fn set_setting(&self, key: &str, value: &str) -> Result<(), AppError> {
        crate::server::set_setting(self, key, value).await
    }

    async fn get_setting(&self, key: &str) -> Result<Option<String>, AppError> {
        crate::server::get_setting(self, key).await
    }

    async fn set_library_root(&self, root: &FolderRef) -> Result<(), AppError> {
        crate::folders::set_library_root(self, root).await
    }

    async fn library_root(&self) -> Result<Option<FolderRef>, AppError> {
        crate::folders::library_root(self).await
    }

    async fn upsert_folder(&self, folder: &FolderRecord) -> Result<(), AppError> {
        crate::folders::upsert_folder(self, folder).await
    }

    async fn folder(&self, id: &str) -> Result<Option<FolderRecord>, AppError> {
        crate::folders::folder(self, id).await
    }

    async fn folder_by_uri(&self, uri: &str) -> Result<Option<FolderRecord>, AppError> {
        crate::folders::folder_by_uri(self, uri).await
    }

    async fn folder_state(&self, id: &str) -> Result<Option<FolderSyncState>, AppError> {
        crate::folders::folder_state(self, id).await
    }

    async fn mark_folder(
        &self,
        id: &str,
        status: &str,
        error: Option<&str>,
        remote_updated_at: Option<i64>,
        now: i64,
    ) -> Result<(), AppError> {
        crate::folders::mark_folder(self, id, status, error, remote_updated_at, now).await
    }

    async fn replace_folder_assets(
        &self,
        folder_id: &str,
        asset_records: &[AssetRecord],
        now: i64,
    ) -> Result<(), AppError> {
        crate::folders::replace_folder_assets(self, folder_id, asset_records, now).await
    }

    async fn retain_folders(&self, keep: &HashSet<String>) -> Result<(), AppError> {
        crate::folders::retain_folders(self, keep).await
    }

    async fn upsert_asset(&self, asset: &AssetRecord) -> Result<(), AppError> {
        crate::assets::upsert_asset(self, asset).await
    }

    async fn move_asset(&self, asset_id: &str, folder_id: &str, uri: &str) -> Result<(), AppError> {
        crate::assets::move_asset(self, asset_id, folder_id, uri).await
    }

    async fn mark_album(&self, folder_id: &str, is_album: bool) -> Result<(), AppError> {
        crate::assets::mark_album(self, folder_id, is_album).await
    }

    async fn rebuild_albums(&self) -> Result<(), AppError> {
        crate::assets::rebuild_albums(self).await
    }

    async fn timeline_page(
        &self,
        cursor: Option<&TimelineCursor>,
        limit: u32,
    ) -> Result<TimelinePage, AppError> {
        crate::assets::timeline_page(self, cursor, limit).await
    }

    async fn album_list(&self) -> Result<Vec<AlbumSummary>, AppError> {
        crate::albums::album_list(self).await
    }

    async fn album(&self, id: &str) -> Result<Option<AlbumSummary>, AppError> {
        crate::albums::album(self, id).await
    }

    async fn album_page(
        &self,
        album_id: &str,
        cursor: Option<&TimelineCursor>,
        limit: u32,
    ) -> Result<TimelinePage, AppError> {
        crate::albums::album_page(self, album_id, cursor, limit).await
    }

    async fn asset(&self, id: &str) -> Result<Option<AssetRecord>, AppError> {
        crate::assets::asset(self, id).await
    }

    async fn set_favorite(&self, asset_id: &str, value: bool) -> Result<(), AppError> {
        crate::assets::set_favorite(self, asset_id, value).await
    }

    async fn set_asset_thumb_hash(&self, asset_id: &str, value: &str) -> Result<(), AppError> {
        crate::assets::set_thumb_hash(self, asset_id, value).await
    }

    async fn set_album_cover(&self, album_id: &str, cover: Option<&str>) -> Result<(), AppError> {
        crate::albums::set_album_cover(self, album_id, cover).await
    }

    async fn set_album_description(
        &self,
        album_id: &str,
        description: Option<&str>,
    ) -> Result<(), AppError> {
        crate::albums::set_album_description(self, album_id, description).await
    }

    async fn search(&self, query: &SearchQuery) -> Result<TimelinePage, AppError> {
        crate::assets::search(self, query).await
    }

    async fn save_local_asset(&self, asset: &LocalAsset) -> Result<(), AppError> {
        crate::local::save_local_asset(self, asset).await
    }

    async fn local_asset(&self, id: &str) -> Result<Option<LocalAsset>, AppError> {
        crate::local::local_asset(self, id).await
    }

    async fn enqueue_upload(&self, task: &UploadTask) -> Result<(), AppError> {
        crate::uploads::enqueue_upload(self, task).await
    }

    async fn update_upload(&self, task: &UploadTask) -> Result<(), AppError> {
        crate::uploads::update_upload(self, task).await
    }

    async fn upload_task(&self, id: &str) -> Result<Option<UploadTask>, AppError> {
        crate::uploads::upload_task(self, id).await
    }

    async fn upload_tasks(&self) -> Result<Vec<UploadTask>, AppError> {
        crate::uploads::upload_tasks(self).await
    }

    async fn pending_uploads(&self) -> Result<Vec<UploadTask>, AppError> {
        crate::uploads::pending_uploads(self).await
    }
}

#[async_trait]
impl UploadSessionPort for SqliteCatalog {
    async fn create_session(&self, key: &str, session: &StoredSession) -> Result<(), AppError> {
        crate::sessions::create_session(self, key, session).await
    }

    async fn append_chunk(&self, key: &str, index: u32, record: &[u8]) -> Result<(), AppError> {
        crate::sessions::append_chunk(self, key, index, record).await
    }

    async fn load_session(&self, key: &str) -> Result<Option<StoredJournal>, AppError> {
        crate::sessions::load_session(self, key).await
    }

    async fn remove_session(&self, key: &str) -> Result<(), AppError> {
        crate::sessions::remove_session(self, key).await
    }
}
