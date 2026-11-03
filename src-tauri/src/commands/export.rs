//! Downloads, exports and cache maintenance.

use tauri::Manager;

use crate::prelude::*;
use source_core::fetch_bytes;

#[tauri::command]
pub(crate) fn cache_stats(state: State<'_, AppState>) -> CacheStats {
    state.cache.stats()
}

#[tauri::command]
pub(crate) fn clear_cache(state: State<'_, AppState>) -> Result<(), AppError> {
    state.cache.clear()
}

#[tauri::command]
pub(crate) async fn export_assets(
    app: AppHandle,
    state: State<'_, AppState>,
    params: ExportAssetsParams,
) -> Result<ExportResult, AppError> {
    let directory = app
        .path()
        .download_dir()
        .unwrap_or_else(|_| state.app_data.join("exports"))
        .join("Source");
    std::fs::create_dir_all(&directory)
        .map_err(|e| AppError::internal(format!("创建导出目录失败: {e}")))?;
    let mut exported = 0u32;
    for id in params.asset_ids {
        let detail = state.core.browse.asset_detail(&id.0).await?;
        let url = state.core.browse.media_url(&id.0).await?;
        let bytes = fetch_bytes(&url).await?;
        let name = sanitize_filename(&detail.download_name);
        let path = unique_path(&directory, &name);
        std::fs::write(&path, bytes)
            .map_err(|e| AppError::internal(format!("写入导出文件失败: {e}")))?;
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
