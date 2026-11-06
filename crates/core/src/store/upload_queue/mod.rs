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
    async fn enqueue_upload(&self, task: &UploadTask) -> Result<(), AppError>;
    async fn update_upload(&self, task: &UploadTask) -> Result<(), AppError>;
    async fn get_upload_task(&self, id: &str) -> Result<Option<UploadTask>, AppError>;
    /// The unfinished part of the queue, newest first.
    async fn list_upload_tasks(&self) -> Result<Vec<UploadTask>, AppError>;
    /// Put in-flight rows back in the queue and report what is waiting.
    async fn recover_uploads(&self) -> Result<Vec<UploadTask>, AppError>;
    /// The local files already waiting for a slot in one destination folder.
    async fn list_queued_local_ids(&self, dest_uri: &str) -> Result<Vec<LocalAssetId>, AppError>;
}

pub(crate) async fn enqueue_upload(cat: &SqliteCatalog, task: &UploadTask) -> Result<(), AppError> {
    let mut conn = cat.conn().await?;
    let error = serialize_error(task.error.as_ref())?;
    let now = now_ms();
    diesel::insert_into(queue::table)
        .values((
            queue::id.eq(task.id.0.clone()),
            queue::local_asset_id.eq(task.local_asset_id.0.clone()),
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

pub(crate) async fn update_upload(cat: &SqliteCatalog, task: &UploadTask) -> Result<(), AppError> {
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
) -> Result<Option<UploadTask>, AppError> {
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
pub(crate) async fn list_upload_tasks(cat: &SqliteCatalog) -> Result<Vec<UploadTask>, AppError> {
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

pub(crate) async fn recover_uploads(cat: &SqliteCatalog) -> Result<Vec<UploadTask>, AppError> {
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
) -> Result<Vec<LocalAssetId>, AppError> {
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

fn serialize_error(error: Option<&AppError>) -> Result<Option<String>, AppError> {
    error
        .map(serde_json::to_string)
        .transpose()
        .map_err(|e| AppError::internal(e.to_string()))
}

#[async_trait]
impl UploadQueueStore for SqliteCatalog {
    async fn enqueue_upload(&self, task: &UploadTask) -> Result<(), AppError> {
        enqueue_upload(self, task).await
    }

    async fn update_upload(&self, task: &UploadTask) -> Result<(), AppError> {
        update_upload(self, task).await
    }

    async fn get_upload_task(&self, id: &str) -> Result<Option<UploadTask>, AppError> {
        get_upload_task(self, id).await
    }

    async fn list_upload_tasks(&self) -> Result<Vec<UploadTask>, AppError> {
        list_upload_tasks(self).await
    }

    async fn recover_uploads(&self) -> Result<Vec<UploadTask>, AppError> {
        recover_uploads(self).await
    }

    async fn list_queued_local_ids(&self, dest_uri: &str) -> Result<Vec<LocalAssetId>, AppError> {
        list_queued_local_ids(self, dest_uri).await
    }
}
