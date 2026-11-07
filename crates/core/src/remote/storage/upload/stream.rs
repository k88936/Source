//! Event-stream upload driving: upload sessions, progress and error mapping.

use tokio_util::sync::CancellationToken;

use super::*;
use crate::remote::UploadUriError;
use crate::store::UploadSessionPort;

/// Map an app-side URI (`file://`, `content://`) onto an SDK upload source.
pub(crate) fn build_upload_source(
    source_uri: &str,
    _size: i64,
) -> Result<UploadSource, UploadUriError> {
    if let Some(path) = source_uri.strip_prefix("file://") {
        return UploadSource::from_path(path).map_err(|error| UploadUriError::SourceUnreadable {
            cause: format!("读取本机文件失败: {error}"),
        });
    }
    if source_uri.starts_with("content://") {
        #[cfg(target_os = "android")]
        {
            return UploadSource::from_content_uri(source_uri, _size.max(0) as u64).map_err(
                |error| UploadUriError::SourceUnreadable {
                    cause: format!("读取 content URI 失败: {error}"),
                },
            );
        }
        #[cfg(not(target_os = "android"))]
        {
            return Err(UploadUriError::SourceUnreadable {
                cause: "当前平台不支持 content:// 上传; 请在 Android 上运行".to_string(),
            });
        }
    }
    Err(UploadUriError::SourceUnreadable {
        cause: format!("不支持的本机文件 URI: {source_uri}"),
    })
}

/// Consume the event stream, recording every finished chunk as it arrives, and
/// report progress to `progress`.
///
/// The session is stored by [`open_upload`] before the first chunk is polled,
/// so an interruption during this attempt can still be resumed; only the
/// finished chunks are appended here. Cancellation is honoured between two
/// chunks, which keeps a cancelled attempt resumable.
pub(crate) async fn drive_upload(
    stream: UploadStream,
    records: Vec<ChunkProgress>,
    sessions: &dyn UploadSessionPort,
    key: &str,
    cancel: &CancellationToken,
    progress: &ProgressSink,
) -> Result<(), UploadUriError> {
    let UploadStream {
        session,
        mut events,
    } = stream;
    // A resumed upload knows its progress from the stored session, before the
    // first chunk of this attempt finishes.
    let mut tracker = Tracker::new(&session, &records);
    progress(upload_progress(tracker.current()));

    while let Some(event) = events.next().await {
        if let Some(update) = tracker.observe(&event) {
            progress(upload_progress(update));
        }
        match event {
            UploadEvent::ChunkDone(record) => {
                // Record first, then stop: a cancelled attempt keeps every
                // chunk the server has already accepted.
                sessions
                    .append_chunk(key, &record)
                    .await
                    .map_err(|cause| UploadUriError::Store { cause })?;
                if cancel.is_cancelled() {
                    return Err(UploadUriError::Interrupted);
                }
            }
            UploadEvent::Completed => {
                let _ = sessions.remove_session(key).await;
                return Ok(());
            }
            UploadEvent::Failed(error) => {
                // The uploader reports a session the server no longer knows as
                // expired before anything new was uploaded. That session can
                // never be resumed, so drop it instead of replaying it on every
                // retry.
                if matches!(error, UploadError::SessionExpired) {
                    let _ = sessions.remove_session(key).await;
                }
                return Err(UploadUriError::from_upload(error));
            }
        }
    }

    Err(UploadUriError::ChunkFailed {
        cause: "上传意外中断".to_string(),
    })
}

/// Open the stream for `params`, resuming the session under `key` when the
/// server session it points at may still be alive.
///
/// A fresh session is stored before the first chunk is polled.
pub(crate) async fn open_upload(
    uploader: &Uploader,
    sessions: &dyn UploadSessionPort,
    key: &str,
    params: UploadParams,
) -> Result<(UploadStream, Vec<ChunkProgress>), UploadUriError> {
    if let Some((session, records)) = sessions
        .load_session(key)
        .await
        .map_err(|cause| UploadUriError::Store { cause })?
    {
        match uploader
            .try_resume(session, params.source.clone(), records.clone())
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
                let _ = sessions.remove_session(key).await;
            }
            Err(error) => return Err(from_upload_rejection(error)),
        }
    }

    let stream = uploader
        .upload(params)
        .await
        .map_err(from_upload_rejection)?;
    sessions
        .create_session(key, &stream.session)
        .await
        .map_err(|cause| UploadUriError::Store { cause })?;
    Ok((stream, Vec::new()))
}

/// The uploader hands back `anyhow`, and only two shapes ever reach this layer:
/// its own error, or the server's answer wrapped with context.
fn from_upload_rejection(error: anyhow::Error) -> UploadUriError {
    match error.downcast::<UploadError>() {
        Ok(error) => UploadUriError::from_upload(error),
        Err(error) => match error.downcast::<cloudreve_sdk_api::ApiError>() {
            Ok(error) => UploadUriError::from_api(error),
            Err(error) => UploadUriError::ChunkFailed {
                cause: error.to_string(),
            },
        },
    }
}

pub(crate) fn upload_progress(update: ProgressUpdate) -> UploadProgress {
    UploadProgress {
        uploaded: update.uploaded as i64,
        total: update.total_size as i64,
        speed: update.speed_bytes_per_sec as i64,
        eta_seconds: update.eta_seconds.map(|v| v as u32),
    }
}
