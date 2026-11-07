//! Content hashing used to detect re-uploads.

use std::fs;
use std::io::Read;
use std::path::Path;

use crate::media::MediaError;
use sha2::{Digest, Sha256};

pub fn sha256_file(path: &Path) -> Result<String, MediaError> {
    let mut file = fs::File::open(path).map_err(|cause| MediaError::Platform {
        cause: format!("open for hash: {cause}"),
    })?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = file
            .read(&mut buf)
            .map_err(|cause| MediaError::Platform {
                cause: format!("hash read: {cause}"),
            })?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
