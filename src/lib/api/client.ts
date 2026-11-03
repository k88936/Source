import type { AppInfo } from "./bindings/AppInfo";
import type { Capacity } from "./bindings/Capacity";
import type { CacheStats } from "./bindings/CacheStats";
import type { AlbumPageParams } from "./bindings/AlbumPageParams";
import type { AlbumSummary } from "./bindings/AlbumSummary";
import type { AssetDetail } from "./bindings/AssetDetail";
import type { AssetDetailParams } from "./bindings/AssetDetailParams";
import type { AssetSummary } from "./bindings/AssetSummary";
import type { EnqueueUploadsParams } from "./bindings/EnqueueUploadsParams";
import type { ExportAssetsParams } from "./bindings/ExportAssetsParams";
import type { ExportResult } from "./bindings/ExportResult";
import type { FolderRef } from "./bindings/FolderRef";
import type { IndexStatus } from "./bindings/IndexStatus";
import type { ListRemoteFoldersParams } from "./bindings/ListRemoteFoldersParams";
import type { LocalAlbum } from "./bindings/LocalAlbum";
import type { LocalAssetPage } from "./bindings/LocalAssetPage";
import type { LocalAssetPageParams } from "./bindings/LocalAssetPageParams";
import type { Login2faParams } from "./bindings/Login2faParams";
import type { LoginOutcome } from "./bindings/LoginOutcome";
import type { LoginParams } from "./bindings/LoginParams";
import type { MediaPermissionState } from "./bindings/MediaPermissionState";
import type { MoveAssetsParams } from "./bindings/MoveAssetsParams";
import type { RemoteFolderPage } from "./bindings/RemoteFolderPage";
import type { SearchParams } from "./bindings/SearchParams";
import type { SessionState } from "./bindings/SessionState";
import type { SetAlbumCoverParams } from "./bindings/SetAlbumCoverParams";
import type { SetAlbumDescriptionParams } from "./bindings/SetAlbumDescriptionParams";
import type { SetFavoriteParams } from "./bindings/SetFavoriteParams";
import type { SetLibraryRootParams } from "./bindings/SetLibraryRootParams";
import type { SetServerParams } from "./bindings/SetServerParams";
import type { SiteInfo } from "./bindings/SiteInfo";
import type { StartIndexParams } from "./bindings/StartIndexParams";
import type { TimelinePage } from "./bindings/TimelinePage";
import type { TimelinePageParams } from "./bindings/TimelinePageParams";
import type { UploadTask } from "./bindings/UploadTask";
import type { UploadTaskParams } from "./bindings/UploadTaskParams";
import { transport } from "./transport";

async function call<T>(command: string, params?: unknown): Promise<T> {
  try {
    return await transport.invoke<T>(
      command,
      params as Record<string, unknown> | undefined,
    );
  } catch (err) {
    if (
      typeof window !== "undefined" &&
      err &&
      typeof err === "object" &&
      "code" in err &&
      (err as { code?: string }).code === "unauthorized"
    ) {
      window.dispatchEvent(new Event("album:unauthorized"));
    }
    throw err;
  }
}

export function listen<T>(event: string, handler: (payload: T) => void) {
  return transport.listen<T>(event, handler);
}

export const api = {
  appInfo: () => call<AppInfo>("app_info"),

  setServer: (params: SetServerParams) => call<SiteInfo>("set_server", params),
  login: (params: LoginParams) => call<LoginOutcome>("login", params),
  login2fa: (params: Login2faParams) => call<SessionState>("login_2fa", params),
  logout: () => call<void>("logout"),
  sessionState: () => call<SessionState | null>("session_state"),
  refreshCapacity: () => call<Capacity>("refresh_capacity"),

  listRemoteFolders: (params: ListRemoteFoldersParams) =>
    call<RemoteFolderPage>("list_remote_folders", params),
  setLibraryRoot: (params: SetLibraryRootParams) =>
    call<FolderRef>("set_library_root", params),

  startIndex: (params: StartIndexParams) =>
    call<IndexStatus>("start_index", params),
  cancelIndex: () => call<IndexStatus>("cancel_index"),
  indexStatus: () => call<IndexStatus>("index_status"),

  timelinePage: (params: TimelinePageParams) =>
    call<TimelinePage>("timeline_page", params),
  albumList: () => call<AlbumSummary[]>("album_list"),
  albumPage: (params: AlbumPageParams) =>
    call<TimelinePage>("album_page", params),
  assetDetail: (params: AssetDetailParams) =>
    call<AssetDetail>("asset_detail", params),
  searchAssets: (params: SearchParams) =>
    call<TimelinePage>("search_assets", params),
  setFavorite: (params: SetFavoriteParams) =>
    call<AssetSummary>("set_favorite", params),
  setAlbumCover: (params: SetAlbumCoverParams) =>
    call<AlbumSummary>("set_album_cover", params),
  setAlbumDescription: (params: SetAlbumDescriptionParams) =>
    call<AlbumSummary>("set_album_description", params),
  moveAssetsToAlbum: (params: MoveAssetsParams) =>
    call<AssetSummary[]>("move_assets_to_album", params),

  mediaPermissionState: () =>
    call<MediaPermissionState>("media_permission_state"),
  requestMediaPermission: () =>
    call<MediaPermissionState>("request_media_permission"),
  localAlbumList: () => call<LocalAlbum[]>("local_album_list"),
  localAssetPage: (params: LocalAssetPageParams) =>
    call<LocalAssetPage>("local_asset_page", params),

  enqueueUploads: (params: EnqueueUploadsParams) =>
    call<UploadTask[]>("enqueue_uploads", params),
  uploadTasks: () => call<UploadTask[]>("upload_tasks"),
  cancelUpload: (params: UploadTaskParams) =>
    call<void>("cancel_upload", params),
  retryUpload: (params: UploadTaskParams) =>
    call<UploadTask>("retry_upload", params),

  cacheStats: () => call<CacheStats>("cache_stats"),
  clearCache: () => call<void>("clear_cache"),
  exportAssets: (params: ExportAssetsParams) =>
    call<ExportResult>("export_assets", params),
};
