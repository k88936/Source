//! The upload queue, persisted so it survives a restart. The contract and the
//! diesel statements behind it.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::dto::*;
use super::rows::*;
use super::schema::upload_queue as queue;
use super::values::*;
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum UploadState {
    Queued,
    CreatingSession,
    Uploading,
    Completing,
    Done,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UploadTask {
    pub id: UploadTaskId,
    pub local_asset_id: String,
    pub name: String,
    pub dest_uri: String,
    pub dest_name: Option<String>,
    pub state: UploadState,
    pub uploaded: i64,
    pub total: i64,
    pub speed: i64,
    pub eta_seconds: Option<u32>,
    pub attempts: u32,
    pub error: Option<AppError>,
}

#[async_trait]
pub trait UploadQueueStore: Send + Sync {
    async fn enqueue_upload(&self, task: &UploadTask) -> Result<(), AppError>;
    async fn update_upload(&self, task: &UploadTask) -> Result<(), AppError>;
    async fn upload_task(&self, id: &str) -> Result<Option<UploadTask>, AppError>;
    async fn upload_tasks(&self) -> Result<Vec<UploadTask>, AppError>;
    async fn pending_uploads(&self) -> Result<Vec<UploadTask>, AppError>;
}

pub(crate) async fn enqueue_upload(cat: &SqliteCatalog, task: &UploadTask) -> Result<(), AppError> {
    let mut conn = cat.conn().await?;
    let error = serialize_error(task.error.as_ref())?;
    let now = now_ms();
    diesel::insert_into(queue::table)
        .values((
            queue::id.eq(task.id.0.clone()),
            queue::local_asset_id.eq(task.local_asset_id.clone()),
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

pub(crate) async fn upload_task(
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

pub(crate) async fn upload_tasks(cat: &SqliteCatalog) -> Result<Vec<UploadTask>, AppError> {
    let mut conn = cat.conn().await?;
    let rows = queue::table
        .select(UploadRow::as_select())
        .order(queue::created_at.desc())
        .load::<UploadRow>(&mut conn)
        .await
        .map_err(db_err)?;
    Ok(rows.into_iter().map(UploadTask::from).collect())
}

pub(crate) async fn pending_uploads(cat: &SqliteCatalog) -> Result<Vec<UploadTask>, AppError> {
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

    async fn upload_task(&self, id: &str) -> Result<Option<UploadTask>, AppError> {
        upload_task(self, id).await
    }

    async fn upload_tasks(&self) -> Result<Vec<UploadTask>, AppError> {
        upload_tasks(self).await
    }

    async fn pending_uploads(&self) -> Result<Vec<UploadTask>, AppError> {
        pending_uploads(self).await
    }
}
