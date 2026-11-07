//! Downloads, exports and cache maintenance.

use tauri::Manager;

use crate::prelude::*;

#[tauri::command]
pub(crate) fn cache_stats(state: State<'_, AppState>) -> CacheStats {
    state.cache.stats()
}

#[tauri::command]
pub(crate) fn clear_cache(state: State<'_, AppState>) -> Result<(), MediaError> {
    state.cache.clear()
}

#[tauri::command]
pub(crate) async fn export_assets(
    app: AppHandle,
    state: State<'_, AppState>,
    params: ExportAssetsParams,
) -> Result<ExportResult, ExportError> {
    let directory = app
        .path()
        .download_dir()
        .unwrap_or_else(|_| state.app_data.join("exports"))
        .join("Source");
    std::fs::create_dir_all(&directory).map_err(|cause| ExportError::DirectoryUnwritable {
        cause: format!("创建导出目录失败: {cause}"),
    })?;
    let mut exported = 0u32;
    for id in params.asset_ids {
        let detail = state
            .core
            .browse
            .get_asset_detail(&id.0)
            .await
            .map_err(|cause| ExportError::Asset { cause })?;
        let bytes = state
            .core
            .browse
            .get_media_bytes(&id.0)
            .await
            .map_err(|cause| ExportError::Asset { cause })?;
        let name = sanitize_filename(&detail.download_name);
        let path = unique_path(&directory, &name);
        std::fs::write(&path, bytes).map_err(|cause| ExportError::WriteFailed {
            path: path.to_string_lossy().to_string(),
            cause: cause.to_string(),
        })?;
        exported += 1;
    }
    Ok(ExportResult {
        exported,
        directory: directory.to_string_lossy().to_string(),
    })
}

pub(crate) fn sanitize_filename(name: &str) -> String {
    let name = PathBuf::from(name)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("asset")
        .to_string();
    name.replace(['/', '\\', '\0'], "_")
}

pub(crate) fn unique_path(directory: &std::path::Path, name: &str) -> PathBuf {
    let candidate = directory.join(name);
    if !candidate.exists() {
        return candidate;
    }
    let stem = std::path::Path::new(name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("asset");
    let ext = std::path::Path::new(name)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    for index in 1..10_000 {
        let file = if ext.is_empty() {
            format!("{stem} ({index})")
        } else {
            format!("{stem} ({index}).{ext}")
        };
        let candidate = directory.join(file);
        if !candidate.exists() {
            return candidate;
        }
    }
    let stamp = source_core::time::now_ms();
    directory.join(format!("{stem}-{stamp}"))
}
