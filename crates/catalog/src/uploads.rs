//! The durable upload queue.

use super::*;
use crate::rows::*;
use crate::schema::upload_queue as queue;
use crate::values::*;

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
