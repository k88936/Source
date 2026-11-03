//! Blur-up placeholder for grid tiles.
//!
//! Cloudreve only serves one thumbnail size, so the placeholder is derived from
//! the JPEG the app already downloads: pixels are box-averaged into a 4×4 RGB
//! mosaic (48 bytes) and base64-encoded. The UI stretches that mosaic behind the
//! real image and blurs it, which looks close to a thumbhash without a second
//! request or an encoder on the server.

use base64::Engine;
use base64::engine::general_purpose::STANDARD;

/// Mosaic width and height in samples.
const GRID: usize = 4;

/// Build a base64 4×4 RGB mosaic from JPEG bytes.
///
/// Returns `None` for anything that is not a decodable RGB/grayscale JPEG: a
/// missing placeholder only costs the blur-up effect, never the tile itself.
pub fn preview_from_jpeg(bytes: &[u8]) -> Option<String> {
    let mut decoder = jpeg_decoder::Decoder::new(bytes);
    let pixels = decoder.decode().ok()?;
    let info = decoder.info()?;
    let width = usize::from(info.width);
    let height = usize::from(info.height);
    if width == 0 || height == 0 {
        return None;
    }
    let channels = pixels.len() / (width * height);
    if channels != 1 && channels != 3 {
        return None;
    }

    let mut mosaic = Vec::with_capacity(GRID * GRID * 3);
    for gy in 0..GRID {
        let y0 = gy * height / GRID;
        let y1 = ((gy + 1) * height / GRID).max(y0 + 1);
        for gx in 0..GRID {
            let x0 = gx * width / GRID;
            let x1 = ((gx + 1) * width / GRID).max(x0 + 1);
            let mut sums = [0u64; 3];
            let mut count = 0u64;
            for y in y0..y1.min(height) {
                for x in x0..x1.min(width) {
                    let offset = (y * width + x) * channels;
                    for channel in 0..3 {
                        // Grayscale JPEGs repeat the single channel.
                        let value = pixels[offset + if channels == 3 { channel } else { 0 }];
                        sums[channel] += u64::from(value);
                    }
                    count += 1;
                }
            }
            if count == 0 {
                return None;
            }
            for sum in sums {
                mosaic.push((sum / count) as u8);
            }
        }
    }

    Some(STANDARD.encode(mosaic))
}
