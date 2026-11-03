//! String encodings for the asset enum columns.

use super::dto::{MediaType, TakenAtSource};

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
