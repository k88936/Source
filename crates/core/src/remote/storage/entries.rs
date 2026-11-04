//! Remote-entry and URI helpers used by the storage implementation.

use cloudreve_sdk_api::models::explorer::FileResponse;

use super::*;

/// Cloudreve signs URLs with its configured `siteURL`. A stock/self-hosted
/// server often reports `http://localhost:5212`, which is meaningless to a
/// phone or another machine. Only rewrite loopback hosts, never CDN URLs.
pub(crate) fn rewrite_loopback_url(base: &str, signed: &str) -> String {
    if signed.starts_with('/') && !signed.starts_with("//") {
        return format!("{}{}", base.trim_end_matches('/'), signed);
    }
    let Ok(base) = reqwest::Url::parse(base) else {
        return signed.to_string();
    };
    let Ok(mut target) = reqwest::Url::parse(signed) else {
        return signed.to_string();
    };
    let loopback = matches!(
        target.host_str(),
        Some("localhost" | "127.0.0.1" | "::1" | "[::1]")
    );
    if !loopback || (target.host_str() == base.host_str() && target.port() == base.port()) {
        return signed.to_string();
    }
    let _ = target.set_scheme(base.scheme());
    let _ = target.set_host(base.host_str());
    let _ = target.set_port(base.port());
    target.to_string()
}

pub(crate) fn join_uri(parent: &str, name: &str) -> String {
    let trimmed = parent.trim_end_matches('/');
    format!("{trimmed}/{}", urlencoding::encode(name))
}

/// The address is always decided by the caller: a listing's `files` are children
/// of the listed folder, while the listing's `parent` field is that folder
/// itself, so neither can be derived from the other.
pub(crate) fn to_entry(uri: &str, file: &FileResponse) -> RemoteEntry {
    let metadata = file.metadata.clone().unwrap_or_default();
    RemoteEntry {
        id: file.id.clone(),
        uri: uri.to_string(),
        name: file.name.clone(),
        path: file.path.clone(),
        is_folder: file.file_type == 1,
        size: file.size,
        created_at: crate::time::parse_rfc3339(&file.created_at).unwrap_or(0),
        updated_at: crate::time::parse_rfc3339(&file.updated_at).unwrap_or(0),
        metadata,
    }
}
