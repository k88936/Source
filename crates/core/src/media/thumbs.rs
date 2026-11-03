//! Disk thumbnail cache: LRU by mtime with a fixed byte budget.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};

use super::CacheStats;
use crate::error::AppError;

pub const THUMB_BUDGET_BYTES: u64 = 512 * 1024 * 1024;

pub struct ThumbCache {
    dir: PathBuf,
    budget: u64,
    lock: Mutex<()>,
}

impl ThumbCache {
    pub fn new(dir: PathBuf, budget: u64) -> Result<Self, AppError> {
        fs::create_dir_all(&dir)
            .map_err(|e| AppError::internal(format!("create thumb cache: {e}")))?;
        Ok(Self {
            dir,
            budget,
            lock: Mutex::new(()),
        })
    }

    pub fn default_cache(app_data: &Path) -> Result<Self, AppError> {
        Self::new(app_data.join("thumbs"), THUMB_BUDGET_BYTES)
    }

    fn path_for(&self, key: &str) -> PathBuf {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        self.dir.join(format!("{:x}.thumb", hasher.finalize()))
    }

    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        let path = self.path_for(key);
        let bytes = fs::read(&path).ok()?;
        // Best effort touch for LRU ordering.
        if let Ok(file) = fs::OpenOptions::new().write(true).open(&path) {
            let _ = file.set_modified(SystemTime::now());
        }
        Some(bytes)
    }

    pub fn put(&self, key: &str, bytes: &[u8]) -> Result<(), AppError> {
        let _guard = self.lock.lock().ok();
        let path = self.path_for(key);
        let tmp = path.with_extension("tmp");
        {
            let mut file = fs::File::create(&tmp)
                .map_err(|e| AppError::internal(format!("write thumb: {e}")))?;
            file.write_all(bytes)
                .map_err(|e| AppError::internal(format!("write thumb: {e}")))?;
        }
        fs::rename(&tmp, &path).map_err(|e| AppError::internal(format!("commit thumb: {e}")))?;
        self.evict_if_needed();
        Ok(())
    }

    pub fn remove(&self, key: &str) {
        let _ = fs::remove_file(self.path_for(key));
    }

    fn scan(&self) -> Vec<(PathBuf, u64, SystemTime)> {
        let Ok(entries) = fs::read_dir(&self.dir) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("thumb") {
                continue;
            }
            let Ok(meta) = entry.metadata() else { continue };
            if !meta.is_file() {
                continue;
            }
            out.push((path, meta.len(), meta.modified().unwrap_or(UNIX_EPOCH)));
        }
        out.sort_by_key(|(_, _, mtime)| *mtime);
        out
    }

    fn evict_if_needed(&self) {
        let mut files = self.scan();
        let mut total: u64 = files.iter().map(|(_, size, _)| *size).sum();
        let mut idx = 0;
        while total > self.budget && idx < files.len() {
            let (path, size, _) = &files[idx];
            if fs::remove_file(path).is_ok() {
                total = total.saturating_sub(*size);
            }
            idx += 1;
        }
        files.truncate(idx);
    }

    pub fn stats(&self) -> CacheStats {
        let files = self.scan();
        let thumb_bytes = files.iter().map(|(_, size, _)| *size as i64).sum();
        CacheStats {
            thumb_files: files.len() as u32,
            thumb_bytes,
            media_bytes: 0,
            media_files: 0,
            budget_bytes: self.budget as i64,
        }
    }

    pub fn clear(&self) -> Result<(), AppError> {
        let files = self.scan();
        for (path, _, _) in files {
            let _ = fs::remove_file(path);
        }
        Ok(())
    }
}
