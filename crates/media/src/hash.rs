//! Content hashing used to detect re-uploads.

use std::fs;
use std::io::Read;
use std::path::Path;

use sha2::{Digest, Sha256};
use source_core::dto::AppError;

pub fn sha256_file(path: &Path) -> Result<String, AppError> {
    let mut file =
        fs::File::open(path).map_err(|e| AppError::internal(format!("open for hash: {e}")))?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = file
            .read(&mut buf)
            .map_err(|e| AppError::internal(format!("hash read: {e}")))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
