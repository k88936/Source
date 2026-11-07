import type {AppInfo} from './bindings/AppInfo';
import type {Capacity} from './bindings/Capacity';
import type {CacheStats} from './bindings/CacheStats';
import type {AddAlbumParams} from './bindings/AddAlbumParams';
import type {AddLocalAlbumParams} from './bindings/AddLocalAlbumParams';
import type {AlbumPageParams} from './bindings/AlbumPageParams';
import type {AlbumParams} from './bindings/AlbumParams';
import type {AlbumSummary} from './bindings/AlbumSummary';
import type {AssetDetail} from './bindings/AssetDetail';
import type {AssetDetailParams} from './bindings/AssetDetailParams';
import type {AssetSummary} from './bindings/AssetSummary';
import type {EnqueueUploadsParams} from './bindings/EnqueueUploadsParams';
import type {ExportAssetsParams} from './bindings/ExportAssetsParams';
import type {ExportResult} from './bindings/ExportResult';
import type {CreateFolderParams} from './bindings/CreateFolderParams';
import type {FolderRef} from './bindings/FolderRef';
import type {ListRemoteFoldersParams} from './bindings/ListRemoteFoldersParams';
import type {LocalAlbumParams} from './bindings/LocalAlbumParams';
import type {LocalAlbumSummary} from './bindings/LocalAlbumSummary';
import type {LocalAssetPage} from './bindings/LocalAssetPage';
import type {LocalAssetPageParams} from './bindings/LocalAssetPageParams';
import type {LocalBrowsePage} from './bindings/LocalBrowsePage';
import type {LocalBrowseParams} from './bindings/LocalBrowseParams';
import type {Login2faParams} from './bindings/Login2faParams';
import type {LoginOutcome} from './bindings/LoginOutcome';
import type {LoginParams} from './bindings/LoginParams';
import type {MediaPermissionState} from './bindings/MediaPermissionState';
import type {MediaSourceParams} from './bindings/MediaSourceParams';
import type {MoveAssetsParams} from './bindings/MoveAssetsParams';
import type {RemoteFolderPage} from './bindings/RemoteFolderPage';
import type {Job} from './bindings/Job';
import type {SearchParams} from './bindings/SearchParams';
import type {AccountState} from './bindings/AccountState';
import type {SetAlbumCoverParams} from './bindings/SetAlbumCoverParams';
import type {SetAlbumDescriptionParams} from './bindings/SetAlbumDescriptionParams';
import type {SetFavoriteParams} from './bindings/SetFavoriteParams';
import type {SetJobsPausedParams} from './bindings/SetJobsPausedParams';
import type {SetServerParams} from './bindings/SetServerParams';
import type {SiteInfo} from './bindings/SiteInfo';
import type {TimelinePage} from './bindings/TimelinePage';
import type {TimelinePageParams} from './bindings/TimelinePageParams';
import type {QueuedUpload} from './bindings/QueuedUpload';
import type {UploadTaskParams} from './bindings/UploadTaskParams';
import {transport} from '#lib/api/transport';

async function call<T>(command: string, params?: unknown): Promise<T> {
    return transport.invoke<T>(command, params as Record<string, unknown> | undefined);
}

export function listen<T>(event: string, handler: (payload: T) => void) {
    return transport.listen<T>(event, handler);
}

export const api = {
    appInfo: () => call<AppInfo>('app_info'),

    setServer: (params: SetServerParams) => call<SiteInfo>('set_server', params),
    login: (params: LoginParams) => call<LoginOutcome>('login', params),
    login2fa: (params: Login2faParams) => call<AccountState>('login_2fa', params),
    logout: () => call<void>('logout'),
    accountState: () => call<AccountState | null>('account_state'),
    capacity: () => call<Capacity>('capacity'),

    listRemoteFolders: (params: ListRemoteFoldersParams) => call<RemoteFolderPage>('list_remote_folders', params),
    createFolder: (params: CreateFolderParams) => call<FolderRef>('create_folder', params),

    timelinePage: (params: TimelinePageParams) => call<TimelinePage>('timeline_page', params),
    albumList: () => call<AlbumSummary[]>('album_list'),
    albumGet: (params: AlbumParams) => call<AlbumSummary | null>('album_get', params),
    albumPage: (params: AlbumPageParams) => call<TimelinePage>('album_page', params),
    addAlbum: (params: AddAlbumParams) => call<AlbumSummary>('album_add', params),
    removeAlbum: (params: AlbumParams) => call<void>('album_remove', params),
    refreshAlbum: (params: AlbumParams) => call<AlbumSummary>('album_refresh', params),
    indexStart: () => call<number>('index_start'),
    indexStop: () => call<void>('index_stop'),
    jobList: () => call<Job[]>('job_list'),
    jobsPaused: () => call<boolean>('jobs_paused'),
    setJobsPaused: (params: SetJobsPausedParams) => call<void>('set_jobs_paused', params),
    assetDetail: (params: AssetDetailParams) => call<AssetDetail>('asset_detail', params),
    mediaSource: (params: MediaSourceParams) => call<string>('media_source', params),
    searchAssets: (params: SearchParams) => call<TimelinePage>('search_assets', params),
    setFavorite: (params: SetFavoriteParams) => call<AssetSummary>('set_favorite', params),
    setAlbumCover: (params: SetAlbumCoverParams) => call<AlbumSummary>('set_album_cover', params),
    setAlbumDescription: (params: SetAlbumDescriptionParams) => call<AlbumSummary>('set_album_description', params),
    moveAssetsToAlbum: (params: MoveAssetsParams) => call<AssetSummary[]>('move_assets_to_album', params),

    mediaPermissionState: () => call<MediaPermissionState>('media_permission_state'),
    requestMediaPermission: () => call<MediaPermissionState>('request_media_permission'),
    localBrowse: (params: LocalBrowseParams) => call<LocalBrowsePage>('local_browse', params),
    localAlbumList: () => call<LocalAlbumSummary[]>('local_album_list'),
    addLocalAlbum: (params: AddLocalAlbumParams) => call<LocalAlbumSummary>('local_album_add', params),
    removeLocalAlbum: (params: LocalAlbumParams) => call<void>('local_album_remove', params),
    localAssetPage: (params: LocalAssetPageParams) => call<LocalAssetPage>('local_asset_page', params),
    syncStart: () => call<number>('sync_start'),
    syncStop: () => call<void>('sync_stop'),

    enqueueUploads: (params: EnqueueUploadsParams) => call<QueuedUpload[]>('enqueue_uploads', params),
    uploadTasks: () => call<QueuedUpload[]>('upload_tasks'),
    cancelUpload: (params: UploadTaskParams) => call<void>('cancel_upload', params),
    retryUpload: (params: UploadTaskParams) => call<QueuedUpload>('retry_upload', params),

    cacheStats: () => call<CacheStats>('cache_stats'),
    clearCache: () => call<void>('clear_cache'),
    exportAssets: (params: ExportAssetsParams) => call<ExportResult>('export_assets', params),
};
