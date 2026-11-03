//! Event-stream upload driving: journal, progress and error mapping.

use std::sync::atomic::{AtomicBool, Ordering};

use super::*;

/// Map an app-side URI (`file://`, `content://`) onto an SDK upload source.
pub(crate) fn build_upload_source(source_uri: &str, _size: i64) -> Result<UploadSource, AppError> {
    if let Some(path) = source_uri.strip_prefix("file://") {
        return UploadSource::from_path(path)
            .map_err(|e| AppError::internal(format!("读取本机文件失败: {e}")));
    }
    if source_uri.starts_with("content://") {
        #[cfg(target_os = "android")]
        {
            return UploadSource::from_content_uri(source_uri, _size.max(0) as u64)
                .map_err(|e| AppError::internal(format!("读取 content URI 失败: {e}")));
        }
        #[cfg(not(target_os = "android"))]
        {
            return Err(AppError::internal(
                "当前平台不支持 content:// 上传；请在 Android 上运行",
            ));
        }
    }
    Err(AppError::internal(format!(
        "不支持的本机文件 URI: {source_uri}"
    )))
}

fn cancelled() -> AppError {
    AppError::new(ErrorCode::Cancelled, "上传已取消".to_string(), false)
}

/// Consume the event stream, journaling every finished chunk as it arrives, and
/// report progress to `progress`.
///
/// The header is written by [`open_upload`] before the first chunk is polled, so
/// an interruption during this attempt can still be resumed; only the finished
/// chunks are appended here. Cancellation is honoured between two chunks, which
/// keeps a cancelled attempt resumable.
pub(crate) async fn drive_upload(
    stream: UploadStream,
    records: Vec<ChunkProgress>,
    journal: &UploadJournal,
    key: &str,
    cancel: &AtomicBool,
    progress: &ProgressSink,
) -> Result<(), AppError> {
    let UploadStream {
        session,
        mut events,
    } = stream;
    // A resumed upload knows its progress from the journal, before the first
    // chunk of this attempt finishes.
    let mut tracker = Tracker::new(&session, &records);
    progress(upload_progress(tracker.current()));

    while let Some(event) = events.next().await {
        if let Some(update) = tracker.observe(&event) {
            progress(upload_progress(update));
        }
        match event {
            UploadEvent::ChunkDone(record) => {
                // Journal first, then stop: a cancelled attempt keeps every
                // chunk the server has already accepted.
                journal.append(key, &record).await?;
                if cancel.load(Ordering::SeqCst) {
                    return Err(cancelled());
                }
            }
            UploadEvent::Completed => {
                journal.remove(key).await;
                return Ok(());
            }
            UploadEvent::Failed(error) => {
                // The uploader reports a session the server no longer knows as
                // expired before anything new was uploaded. That journal can
                // never be resumed, so drop it instead of replaying it on every
                // retry.
                if matches!(error, UploadError::SessionExpired) {
                    journal.remove(key).await;
                }
                return Err(map_upload_error(error.into()));
            }
        }
    }

    Err(AppError::internal("上传意外中断"))
}

/// Open the stream for `params`, resuming the journal under `key` when the
/// server session it points at may still be alive.
///
/// A fresh session has its header journaled before the first chunk is polled.
pub(crate) async fn open_upload(
    uploader: &Uploader,
    journal: &UploadJournal,
    key: &str,
    params: UploadParams,
) -> Result<(UploadStream, Vec<ChunkProgress>), AppError> {
    if let Some((header, records)) = journal.load(key).await {
        match uploader
            .try_resume(header.session(), params.source.clone(), records.clone())
            .await
        {
            Ok(stream) => return Ok((stream, records)),
            // The recorded session expired before this attempt could use it.
            // Nothing was uploaded, so start over with a fresh one instead of
            // reporting a failure the user cannot act on.
            Err(error)
                if matches!(
                    error.downcast_ref::<UploadError>(),
                    Some(UploadError::SessionExpired)
                ) =>
            {
                journal.remove(key).await;
            }
            Err(error) => return Err(map_upload_error(error)),
        }
    }

    let stream = uploader.upload(params).await.map_err(map_upload_error)?;
    journal.create(key, &stream.session).await?;
    Ok((stream, Vec::new()))
}

pub(crate) fn upload_progress(update: ProgressUpdate) -> UploadProgress {
    UploadProgress {
        uploaded: update.uploaded as i64,
        total: update.total_size as i64,
        speed: update.speed_bytes_per_sec as i64,
        eta_seconds: update.eta_seconds.map(|v| v as u32),
    }
}

pub(crate) fn map_upload_error(error: anyhow::Error) -> AppError {
    match error.downcast::<UploadError>() {
        Ok(error) => map_upload_failure(&error),
        Err(error) => {
            let message = error.to_string();
            AppError::new(
                ErrorCode::Server,
                format!("上传失败: {message}"),
                is_transient(&message),
            )
        }
    }
}

pub(crate) fn map_upload_failure(error: &UploadError) -> AppError {
    // The uploader reports a session the server no longer knows as expired, and
    // the journal was dropped, so a retry starts a fresh session from zero.
    if matches!(error, UploadError::SessionExpired) {
        return AppError::new(
            ErrorCode::Server,
            "上传会话已在服务器端失效，请重试".to_string(),
            true,
        );
    }
    let message = error.to_string();
    AppError::new(
        ErrorCode::Server,
        format!("上传失败: {message}"),
        is_transient(&message),
    )
}

pub(crate) fn is_transient(message: &str) -> bool {
    message.contains("timeout") || message.contains("connection") || message.contains("429")
}
