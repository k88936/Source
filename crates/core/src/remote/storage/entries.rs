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

pub(crate) fn to_entry(parent_uri: &str, file: &FileResponse) -> RemoteEntry {
    let uri = join_uri(parent_uri, &file.name);
    let metadata = file.metadata.clone().unwrap_or_default();
    RemoteEntry {
        id: file.id.clone(),
        uri,
        name: file.name.clone(),
        path: file.path.clone(),
        is_folder: file.file_type == 1,
        size: file.size,
        created_at: crate::time::parse_rfc3339(&file.created_at).unwrap_or(0),
        updated_at: crate::time::parse_rfc3339(&file.updated_at).unwrap_or(0),
        metadata,
    }
}

pub(crate) fn is_root_uri(uri: &str) -> bool {
    uri.trim_end_matches('/') == "cloudreve://my"
}

pub(crate) fn root_entry(uri: &str, file: Option<&FileResponse>) -> RemoteEntry {
    RemoteEntry {
        id: file
            .map(|f| f.id.clone())
            .unwrap_or_else(|| "root".to_string()),
        uri: if uri.ends_with('/') {
            uri.to_string()
        } else {
            format!("{uri}/")
        },
        name: file
            .map(|f| f.name.clone())
            .unwrap_or_else(|| "我的文件".to_string()),
        path: file
            .map(|f| f.path.clone())
            .filter(|path| !path.is_empty())
            .unwrap_or_else(|| "cloudreve://my/".to_string()),
        is_folder: true,
        size: 0,
        created_at: file
            .map(|f| crate::time::parse_rfc3339(&f.created_at).unwrap_or(0))
            .unwrap_or(0),
        updated_at: file
            .map(|f| crate::time::parse_rfc3339(&f.updated_at).unwrap_or(0))
            .unwrap_or(0),
        metadata: file.and_then(|f| f.metadata.clone()).unwrap_or_default(),
    }
}
