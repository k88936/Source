//! The upload queue, persisted so it survives a restart. The contract and the
//! diesel statements behind it.

use async_trait::async_trait;

use crate::time::now_ms;

use self::schema::upload_queue as queue;
use super::*;

mod dto;
pub(crate) mod rows;
pub(crate) mod schema;
mod values;

pub use dto::*;

use self::rows::*;
use self::values::*;

/// How many unfinished rows the queue card reads at once.
const QUEUE_PAGE: i64 = 200;

#[async_trait]
pub trait UploadQueueStore: Send + Sync {
    async fn enqueue_upload(&self, task: &UploadTask) -> Result<(), CatalogError>;
    async fn update_upload(&self, task: &UploadTask) -> Result<(), CatalogError>;
    async fn get_upload_task(&self, id: &str) -> Result<Option<UploadTask>, CatalogError>;
    /// The unfinished part of the queue, newest first.
    async fn list_upload_tasks(&self) -> Result<Vec<UploadTask>, CatalogError>;
    /// Put in-flight rows back in the queue and report what is waiting.
    async fn recover_uploads(&self) -> Result<Vec<UploadTask>, CatalogError>;
    /// The local files already waiting for a slot in one destination folder.
    async fn list_queued_local_ids(
        &self,
        dest_uri: &str,
    ) -> Result<Vec<LocalAssetId>, CatalogError>;
    /// The oldest rows still waiting for a slot, oldest first.
    async fn next_queued(&self, limit: i64) -> Result<Vec<UploadTask>, CatalogError>;
}

pub(crate) async fn enqueue_upload(
    cat: &SqliteCatalog,
    task: &UploadTask,
) -> Result<(), CatalogError> {
    let mut conn = cat.conn().await?;
    let error = serialize_error(task.error.as_ref())?;
    let now = now_ms();
    diesel::insert_into(queue::table)
        .values((
            queue::id.eq(task.id.0.clone()),
            queue::local_asset_id.eq(task.local_asset_id.0.clone()),
            queue::local_album_id.eq(task.local_album_id.clone()),
            queue::dest_uri.eq(task.dest_uri.clone()),
            queue::dest_name.eq(task.dest_name.clone()),
            queue::name.eq(task.name.clone()),
            queue::state.eq(upload_state_to_str(task.state)),
            queue::uploaded.eq(task.uploaded),
            queue::total.eq(task.total),
            queue::speed.eq(task.speed),
            queue::eta_seconds.eq(task.eta_seconds.map(i64::from)),
            queue::attempts.eq(i64::from(task.attempts)),
            queue::error.eq(error),
            queue::created_at.eq(now),
            queue::updated_at.eq(now),
        ))
        .execute(&mut conn)
        .await
        .map_err(db_err)?;
    Ok(())
}

pub(crate) async fn update_upload(
    cat: &SqliteCatalog,
    task: &UploadTask,
) -> Result<(), CatalogError> {
    let mut conn = cat.conn().await?;
    let error = serialize_error(task.error.as_ref())?;
    diesel::update(queue::table.filter(queue::id.eq(task.id.0.clone())))
        .set((
            queue::state.eq(upload_state_to_str(task.state)),
            queue::uploaded.eq(task.uploaded),
            queue::total.eq(task.total),
            queue::speed.eq(task.speed),
            queue::eta_seconds.eq(task.eta_seconds.map(i64::from)),
            queue::attempts.eq(i64::from(task.attempts)),
            queue::error.eq(error),
            queue::updated_at.eq(now_ms()),
        ))
        .execute(&mut conn)
        .await
        .map_err(db_err)?;
    Ok(())
}

pub(crate) async fn get_upload_task(
    cat: &SqliteCatalog,
    id: &str,
) -> Result<Option<UploadTask>, CatalogError> {
    let mut conn = cat.conn().await?;
    queue::table
        .filter(queue::id.eq(id))
        .select(UploadRow::as_select())
        .first::<UploadRow>(&mut conn)
        .await
        .optional()
        .map(|row| row.map(UploadTask::from))
        .map_err(db_err)
}

/// The queue still moving, newest first, capped at [`QUEUE_PAGE`].
///
/// A first sync can enqueue tens of thousands of files; the queue card shows
/// what is left, not every row ever written, so this never grows with the
/// history and never hands the UI a payload it cannot render.
pub(crate) async fn list_upload_tasks(
    cat: &SqliteCatalog,
) -> Result<Vec<UploadTask>, CatalogError> {
    let mut conn = cat.conn().await?;
    let rows = queue::table
        .filter(queue::state.ne(upload_state_to_str(UploadState::Done)))
        .filter(queue::state.ne(upload_state_to_str(UploadState::Cancelled)))
        .select(UploadRow::as_select())
        .order(queue::created_at.desc())
        .limit(QUEUE_PAGE)
        .load::<UploadRow>(&mut conn)
        .await
        .map_err(db_err)?;
    Ok(rows.into_iter().map(UploadTask::from).collect())
}

pub(crate) async fn recover_uploads(cat: &SqliteCatalog) -> Result<Vec<UploadTask>, CatalogError> {
    let mut conn = cat.conn().await?;
    // A previous process may have died mid-upload. Requeue in-flight rows
    // so startup actually loads the durable session snapshot and resumes.
    diesel::update(queue::table.filter(queue::state.eq_any([
        "creatingSession",
        "uploading",
        "completing",
    ])))
    .set((
        queue::state.eq("queued"),
        queue::speed.eq(0),
        queue::eta_seconds.eq(None::<i64>),
        queue::error.eq(None::<String>),
    ))
    .execute(&mut conn)
    .await
    .map_err(db_err)?;

    let rows = queue::table
        .filter(queue::state.eq("queued"))
        .select(UploadRow::as_select())
        .order(queue::created_at.asc())
        .load::<UploadRow>(&mut conn)
        .await
        .map_err(db_err)?;
    Ok(rows.into_iter().map(UploadTask::from).collect())
}

/// The local files already waiting for a slot in one destination folder.
pub(crate) async fn list_queued_local_ids(
    cat: &SqliteCatalog,
    dest_uri: &str,
) -> Result<Vec<LocalAssetId>, CatalogError> {
    let mut conn = cat.conn().await?;
    queue::table
        .filter(queue::dest_uri.eq(dest_uri))
        .filter(queue::state.eq_any(["queued", "creatingSession", "uploading", "completing"]))
        .select(queue::local_asset_id)
        .load::<String>(&mut conn)
        .await
        .map(|ids| ids.into_iter().map(LocalAssetId).collect())
        .map_err(db_err)
}

/// The oldest rows still waiting for a slot, oldest first.
///
/// The table is the backlog: this is how the upload service learns what to
/// start next, so a restart, a retry or a finished job all resume from here.
pub(crate) async fn next_queued(
    cat: &SqliteCatalog,
    limit: i64,
) -> Result<Vec<UploadTask>, CatalogError> {
    let mut conn = cat.conn().await?;
    let rows = queue::table
        .filter(queue::state.eq(upload_state_to_str(UploadState::Queued)))
        .select(UploadRow::as_select())
        .order(queue::created_at.asc())
        .limit(limit)
        .load::<UploadRow>(&mut conn)
        .await
        .map_err(db_err)?;
    Ok(rows.into_iter().map(UploadTask::from).collect())
}

fn serialize_error(error: Option<&String>) -> Result<Option<String>, CatalogError> {
    error.cloned().map(Ok).transpose()
}

#[async_trait]
impl UploadQueueStore for SqliteCatalog {
    async fn enqueue_upload(&self, task: &UploadTask) -> Result<(), CatalogError> {
        enqueue_upload(self, task).await
    }

    async fn update_upload(&self, task: &UploadTask) -> Result<(), CatalogError> {
        update_upload(self, task).await
    }

    async fn get_upload_task(&self, id: &str) -> Result<Option<UploadTask>, CatalogError> {
        get_upload_task(self, id).await
    }

    async fn list_upload_tasks(&self) -> Result<Vec<UploadTask>, CatalogError> {
        list_upload_tasks(self).await
    }

    async fn recover_uploads(&self) -> Result<Vec<UploadTask>, CatalogError> {
        recover_uploads(self).await
    }

    async fn list_queued_local_ids(
        &self,
        dest_uri: &str,
    ) -> Result<Vec<LocalAssetId>, CatalogError> {
        list_queued_local_ids(self, dest_uri).await
    }

    async fn next_queued(&self, limit: i64) -> Result<Vec<UploadTask>, CatalogError> {
        next_queued(self, limit).await
    }
}
