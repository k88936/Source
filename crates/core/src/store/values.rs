//! String encodings for the enum columns.

use super::*;

// ---------------------------------------------------------------------------
// Value conversions

pub(crate) fn media_type_to_str(value: MediaType) -> &'static str {
    match value {
        MediaType::Image => "image",
        MediaType::Video => "video",
        MediaType::Audio => "audio",
        MediaType::Other => "other",
    }
}

pub(crate) fn media_type_from_str(value: &str) -> MediaType {
    match value {
        "image" => MediaType::Image,
        "video" => MediaType::Video,
        "audio" => MediaType::Audio,
        _ => MediaType::Other,
    }
}

pub(crate) fn taken_source_to_str(value: TakenAtSource) -> &'static str {
    match value {
        TakenAtSource::Exif => "exif",
        TakenAtSource::ClientExif => "clientExif",
        TakenAtSource::UpdatedAt => "updatedAt",
        TakenAtSource::Filename => "filename",
    }
}

pub(crate) fn taken_source_from_str(value: &str) -> TakenAtSource {
    match value {
        "exif" => TakenAtSource::Exif,
        "clientExif" => TakenAtSource::ClientExif,
        "updatedAt" => TakenAtSource::UpdatedAt,
        _ => TakenAtSource::Filename,
    }
}

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

pub(crate) use crate::time::now_ms;
