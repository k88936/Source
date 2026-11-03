import { convertFileSrc } from "@tauri-apps/api/core";

export function thumbUrl(assetId: string): string {
  return convertFileSrc(assetId, "thumb");
}

export function mediaUrl(assetId: string): string {
  return convertFileSrc(assetId, "media");
}

export function localThumb(asset: { id: string; uri: string }): string {
  return convertFileSrc(asset.uri, "local");
}
