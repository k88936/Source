//! String encodings for the upload state column.

use super::dto::UploadState;

pub(crate) fn upload_state_to_str(value: UploadState) -> &'static str {
    match value {
        UploadState::Queued => "queued",
        UploadState::CreatingSession => "creatingSession",
        UploadState::Uploading => "uploading",
        UploadState::Completing => "completing",
        UploadState::Done => "done",
        UploadState::Failed => "failed",
        UploadState::Cancelled => "cancelled",
    }
}

pub(crate) fn upload_state_from_str(value: &str) -> UploadState {
    match value {
        "creatingSession" => UploadState::CreatingSession,
        "uploading" => UploadState::Uploading,
        "completing" => UploadState::Completing,
        "done" => UploadState::Done,
        "failed" => UploadState::Failed,
        "cancelled" => UploadState::Cancelled,
        _ => UploadState::Queued,
    }
}
