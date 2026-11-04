/**
 * Blur-up placeholder for a tile, built from the 4x4 RGB mosaic the backend
 * stores in `AssetSummary.thumbHash` (base64, 48 bytes).
 *
 * The mosaic is expanded into a tiny inline SVG and stretched behind the real
 * image with a blur, which reads as a smooth gradient - close enough to a
 * thumbhash without shipping a decoder to the webview.
 */

const cache = new Map<string, string | null>();

export function mosaicBackground(hash: string | null): string | null {
  if (!hash) return null;
  const cached = cache.get(hash);
  if (cached !== undefined) return cached;

  let url: string | null = null;
  try {
    const bytes = atob(hash);
    if (bytes.length === 48) {
      let rects = "";
      for (let y = 0; y < 4; y += 1) {
        for (let x = 0; x < 4; x += 1) {
          const offset = (y * 4 + x) * 3;
          const color = `rgb(${bytes.charCodeAt(offset)},${bytes.charCodeAt(offset + 1)},${bytes.charCodeAt(offset + 2)})`;
          rects += `<rect x="${x}" y="${y}" width="1" height="1" fill="${color}"/>`;
        }
      }
      const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 4 4" preserveAspectRatio="none">${rects}</svg>`;
      url = `url("data:image/svg+xml,${encodeURIComponent(svg)}")`;
    }
  } catch {
    url = null;
  }
  cache.set(hash, url);
  return url;
}
