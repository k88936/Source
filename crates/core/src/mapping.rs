//! Pure mappings from Cloudreve navigator entries to domain records.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::json;
use ts_rs::TS;

use crate::exif::ExifData;
use crate::remote::RemoteEntry;
use crate::store::{AssetId, AssetRecord, AssetSummary, FolderId, MediaType, TakenAtSource};
use crate::time::{parse_filename_date, parse_rfc3339};

pub const FAVORITE_KEY: &str = "customize:album_favorite";
pub const COVER_KEY: &str = "customize:album_cover";
pub const DESCRIPTION_KEY: &str = "customize:album_description";

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ExifField {
    pub key: String,
    pub label: String,
    pub value: String,
}

pub fn metadata_bool(metadata: &HashMap<String, String>, key: &str) -> bool {
    metadata
        .get(key)
        .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "yes"))
        .unwrap_or(false)
}

/// MIME type for a file name, `None` when the extension is unknown.
pub fn mime_for(name: &str) -> Option<&'static str> {
    mime_guess::from_path(name).first_raw()
}

/// Classify an entry by the MIME type the server reported, else by its name.
pub fn media_type(mime: Option<&str>, name: &str) -> MediaType {
    let mime = mime.or_else(|| mime_for(name));
    match mime.map(str::to_ascii_lowercase) {
        Some(mime) if mime.starts_with("image/") => MediaType::Image,
        Some(mime) if mime.starts_with("video/") => MediaType::Video,
        Some(mime) if mime.starts_with("audio/") => MediaType::Audio,
        _ => MediaType::Other,
    }
}

/// Classify a remote entry by the MIME type the server reported, else its name.
pub fn entry_media_type(entry: &RemoteEntry) -> MediaType {
    media_type(
        entry.metadata.get("sys:mime").map(String::as_str),
        &entry.name,
    )
}

fn parse_number(raw: &str) -> Option<i64> {
    let raw = raw.trim();
    if let Ok(v) = raw.parse::<i64>() {
        return Some(v);
    }
    raw.parse::<f64>().ok().map(|v| v as i64)
}

fn parse_timestamp(raw: &str) -> Option<i64> {
    if let Some(ts) = parse_rfc3339(raw) {
        return Some(ts);
    }
    // Some servers expose Unix seconds. Values in ms are already large.
    parse_number(raw).map(|v| if v > 10_000_000_000 { v } else { v * 1000 })
}

fn meta_i32(metadata: &HashMap<String, String>, keys: &[&str]) -> Option<i32> {
    keys.iter()
        .find_map(|k| metadata.get(*k).and_then(|v| parse_number(v)))
        .filter(|v| *v > 0 && *v < i32::MAX as i64)
        .map(|v| v as i32)
}

pub fn duration_ms(metadata: &HashMap<String, String>) -> Option<i64> {
    let raw = ["stream:duration", "video:duration", "music:duration"]
        .iter()
        .find_map(|k| metadata.get(*k))?;
    let value = raw.trim().parse::<f64>().ok()?;
    // Cloudreve's stream:duration is seconds; tolerate ms-sized values.
    Some(if value > 100_000.0 {
        value as i64
    } else {
        (value * 1000.0) as i64
    })
}

/// Capture-time fallback chain:
/// `exif:taken_at` → client EXIF → `updated_at` → filename date.
pub fn infer_taken_at(
    metadata: &HashMap<String, String>,
    client_exif: Option<i64>,
    updated_at: i64,
    name: &str,
) -> (i64, TakenAtSource) {
    if let Some(raw) = metadata.get("exif:taken_at")
        && let Some(ts) = parse_timestamp(raw)
    {
        return (ts, TakenAtSource::Exif);
    }
    if let Some(ts) = client_exif {
        return (ts, TakenAtSource::ClientExif);
    }
    if updated_at > 0 {
        return (updated_at, TakenAtSource::UpdatedAt);
    }
    if let Some(ts) = parse_filename_date(name)
        && ts > 0
    {
        return (ts, TakenAtSource::Filename);
    }
    (0, TakenAtSource::Filename)
}

pub fn asset_record(
    entry: &RemoteEntry,
    folder_id: &FolderId,
    client_exif: Option<&ExifData>,
) -> AssetRecord {
    let mime = entry
        .metadata
        .get("sys:mime")
        .cloned()
        .or_else(|| mime_for(&entry.name).map(str::to_owned));
    let mt = entry_media_type(entry);
    let exif_taken = client_exif.and_then(|e| e.taken_at);
    let (taken_at, taken_at_source) =
        infer_taken_at(&entry.metadata, exif_taken, entry.updated_at, &entry.name);

    let width = meta_i32(
        &entry.metadata,
        &[
            "exif:width",
            "exif:x",
            "image:width",
            "stream:width",
            "props:width",
        ],
    )
    .or_else(|| client_exif.and_then(|e| e.width));
    let height = meta_i32(
        &entry.metadata,
        &[
            "exif:height",
            "exif:y",
            "image:height",
            "stream:height",
            "props:height",
        ],
    )
    .or_else(|| client_exif.and_then(|e| e.height));

    let mut exif = serde_json::Map::new();
    for (key, value) in &entry.metadata {
        if key.starts_with("exif:") || key.starts_with("stream:") || key.starts_with("music:") {
            exif.insert(key.clone(), json!(value));
        }
    }

    AssetRecord {
        summary: AssetSummary {
            id: AssetId::new(entry.id.clone()),
            folder_id: folder_id.clone(),
            name: entry.name.clone(),
            media_type: mt,
            mime,
            size: entry.size,
            taken_at,
            taken_at_source,
            width,
            height,
            duration_ms: duration_ms(&entry.metadata),
            favorite: metadata_bool(&entry.metadata, FAVORITE_KEY),
            // Filled lazily by the thumb protocol; never known at scan time.
            thumb_hash: None,
        },
        uri: entry.uri.clone(),
        exif_json: if exif.is_empty() {
            None
        } else {
            Some(serde_json::Value::Object(exif).to_string())
        },
        taken_at_source,
        remote_updated_at: entry.updated_at,
    }
}

pub fn exif_fields(raw: Option<&str>) -> Vec<ExifField> {
    let Some(raw) = raw else {
        return Vec::new();
    };
    let Ok(serde_json::Value::Object(map)) = serde_json::from_str::<serde_json::Value>(raw) else {
        return Vec::new();
    };
    let mut fields: Vec<ExifField> = map
        .into_iter()
        .filter_map(|(key, value)| {
            let value = value.as_str()?.to_string();
            if value.is_empty() {
                return None;
            }
            Some(ExifField {
                label: exif_label(&key),
                key,
                value,
            })
        })
        .collect();
    fields.sort_by(|a, b| a.key.cmp(&b.key));
    fields
}

fn exif_label(key: &str) -> String {
    let short = key.split_once(':').map(|(_, v)| v).unwrap_or(key);
    let label = match short {
        "taken_at" => "拍摄时间",
        "camera_model" => "相机型号",
        "camera_make" => "相机品牌",
        "lens_model" => "镜头",
        "iso" => "ISO",
        "f" => "光圈",
        "exposure_time" => "快门",
        "focal_length" => "焦距",
        "width" => "宽度",
        "height" => "高度",
        "orientation" => "方向",
        "duration" => "时长",
        "bitrate" => "码率",
        "fps" => "帧率",
        "codec" => "编码",
        "latitude" => "纬度",
        "longitude" => "经度",
        "altitude" => "海拔",
        "artist" => "作者",
        "copyright" => "版权",
        "software" => "软件",
        "flash" => "闪光灯",
        "x" => "宽度",
        "y" => "高度",
        _ => short,
    };
    if label == short {
        short.to_string()
    } else {
        format!("{label} ({short})")
    }
}
