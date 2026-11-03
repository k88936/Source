//! Thumbnails for local files that never went through the Cloudreve catalog.
//!
//! The grid used to load original bytes for every tile, so a folder of full
//! resolution screenshots made the webview decode megabytes per tile and made
//! the protocol thread block while reading them. One cached JPEG per file keeps
//! tiles cheap to paint.

use std::path::Path;
use std::sync::Mutex;

use image::ImageReader;
use image::codecs::jpeg::JpegEncoder;

/// Longest edge, in pixels, of a generated local thumbnail.
pub const LOCAL_THUMB_EDGE: u32 = 512;

const QUALITY: u8 = 78;

/// Frames are cut by `ffmpeg`; one extraction at a time keeps a fast scroll
/// from forking a process per tile.
static VIDEO_EXTRACT: Mutex<()> = Mutex::new(());

/// Decode `path` and re-encode it as a small JPEG.
///
/// `None` covers anything the decoder does not understand (HEIC, RAW, corrupt
/// files), so the caller can answer without streaming the original.
pub fn jpeg_thumb_from_path(path: &Path, edge: u32) -> Option<Vec<u8>> {
    let img = ImageReader::open(path)
        .ok()?
        .with_guessed_format()
        .ok()?
        .decode()
        .ok()?;
    let thumb = img.thumbnail(edge, edge);
    let mut out = Vec::new();
    JpegEncoder::new_with_quality(&mut out, QUALITY)
        .encode_image(&thumb)
        .ok()?;
    Some(out)
}

/// Grab one frame of `path` as a JPEG through the system `ffmpeg`.
///
/// `None` when ffmpeg is unavailable or the file cannot be demuxed, so the
/// caller can answer without touching the video itself.
pub fn video_thumb_from_path(path: &Path, edge: u32) -> Option<Vec<u8>> {
    let _guard = VIDEO_EXTRACT.lock().ok();
    let output = std::process::Command::new("ffmpeg")
        .args(["-v", "error", "-nostdin", "-ss", "0", "-i"])
        .arg(path)
        .args([
            "-frames:v",
            "1",
            "-vf",
            &format!("scale={edge}:-2"),
            "-f",
            "mjpeg",
            "-q:v",
            "4",
            "-",
        ])
        .output()
        .ok()?;
    if !output.status.success() || output.stdout.len() < 1024 {
        return None;
    }
    Some(output.stdout)
}
