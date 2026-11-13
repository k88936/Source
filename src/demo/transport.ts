import type {Transport} from '../lib/api/transport';
import type {AlbumMetadata} from '#lib/api/bindings/AlbumMetadata';
import type {AlbumParams} from '#lib/api/bindings/AlbumParams';
import type {AddAlbumParams} from '#lib/api/bindings/AddAlbumParams';
import type {AddLocalAlbumParams} from '#lib/api/bindings/AddLocalAlbumParams';
import type {AssetCountParams} from '#lib/api/bindings/AssetCountParams';
import type {AssetEdgeParams} from '#lib/api/bindings/AssetEdgeParams';
import type {AssetSummary} from '#lib/api/bindings/AssetSummary';
import type {AssetsQuery} from '#lib/api/bindings/AssetsQuery';
import type {CreateFolderParams} from '#lib/api/bindings/CreateFolderParams';
import type {ExportAssetsParams} from '#lib/api/bindings/ExportAssetsParams';
import type {LocalAssetPage} from '#lib/api/bindings/LocalAssetPage';
import type {LocalAssetPageParams} from '#lib/api/bindings/LocalAssetPageParams';
import type {LocalAsset} from '#lib/api/bindings/LocalAsset';
import type {LocalBrowsePage} from '#lib/api/bindings/LocalBrowsePage';
import type {LoginOutcome} from '#lib/api/bindings/LoginOutcome';
import type {MediaSourceParams} from '#lib/api/bindings/MediaSourceParams';
import type {RemoteFolderPage} from '#lib/api/bindings/RemoteFolderPage';
import type {SearchParams} from '#lib/api/bindings/SearchParams';
import type {SetAlbumCoverParams} from '#lib/api/bindings/SetAlbumCoverParams';
import type {SetAlbumDescriptionParams} from '#lib/api/bindings/SetAlbumDescriptionParams';
import type {SetFavoriteParams} from '#lib/api/bindings/SetFavoriteParams';
import type {TimelineCursor} from '#lib/api/bindings/TimelineCursor';
import type {TimelinePage} from '#lib/api/bindings/TimelinePage';
import type {ClientMessage, ServerMessage} from 'subscription-client';
import * as demo from './fixtures';

function paged(items: AssetSummary[], cursor: TimelineCursor | null, limit: number): TimelinePage {
    const from = cursor ? items.findIndex((item) => item.id === cursor.id) + 1 : 0;
    const page = items.slice(from, from + limit);
    if (from + limit >= items.length) return {items: page, next: null, total: items.length};
    const last = page[page.length - 1]!;
    return {items: page, next: {takenAt: last.takenAt, id: last.id}, total: items.length};
}

function pagedLocal(items: LocalAsset[], cursor: string | null, limit: number): LocalAssetPage {
    const from = cursor ? Number(cursor) : 0;
    const page = items.slice(from, from + limit);
    return {items: page, next: from + limit < items.length ? String(from + limit) : null};
}

/**
 * One scope of the timeline, newest first.
 *
 * An asset window is an interval: `from` inclusive, `to` exclusive, so the
 * fixture set answers exactly like the catalog does.
 */
function orderedAssets(query: AssetsQuery): AssetSummary[] {
    const all = query.albumId === null ? demo.assets : demo.albumAssets(query.albumId);
    return [...all]
        .sort((a, b) => b.takenAt - a.takenAt || b.id.localeCompare(a.id))
        .filter((asset) => {
            const key = cursorOf(asset);
            return (!query.from || !isOlder(key, query.from)) && (!query.to || isOlder(key, query.to));
        });
}

function cursorOf(asset: AssetSummary): TimelineCursor {
    return {takenAt: asset.takenAt, id: asset.id};
}

function isOlder(a: TimelineCursor, b: TimelineCursor): boolean {
    return a.takenAt < b.takenAt || (a.takenAt === b.takenAt && a.id < b.id);
}

/** Pages subscribed to a view, one set per view name. */
const subscribers = new Map<string, Set<(payload: unknown) => void>>();

/**
 * The demo runs the real app against fixtures instead of the Rust side. Writes
 * answer the way the adapter does, but change nothing: the point is to show the
 * screens, not to keep a database. The visitor is always signed in, so the demo
 * opens on the photos page instead of the welcome screen.
 */
function run(command: string, params: Record<string, unknown>): unknown {
    switch (command) {
        case 'app_info':
            return demo.appInfo;

        case 'set_server':
            return {name: 'Source Demo', thumbnailSize: 256};

        case 'login': {
            const outcome: LoginOutcome = {kind: 'authenticated', account: demo.account};
            return outcome;
        }

        case 'login_2fa':
            return demo.account;

        case 'logout':
            return undefined;

        case 'session_state':
            return {account: demo.account, setup: 'live'};

        case 'capacity':
            return demo.capacity;

        case 'cache_stats':
            return demo.cacheStats;

        case 'clear_cache':
        case 'set_keep_alive':
        case 'index_stop':
        case 'sync_stop':
            return undefined;

        case 'index_start':
        case 'sync_start':
            return 0;

        case 'keep_alive':
            return true;

        case 'job_list':
            return demo.jobs;

        case 'list_remote_folders': {
            const {uri} = params as {uri: string};
            const page: RemoteFolderPage = {
                folders: demo.foldersInside(uri),
                current: demo.folder(uri),
                next: null,
            };
            return page;
        }

        case 'create_folder': {
            const {parentUri, name} = params as CreateFolderParams;
            const parent = demo.folder(parentUri);
            const uri = `${parent.uri}/${name}`;
            return {id: `folder-${name}`, uri, name, path: `${parent.path}/${name}`};
        }

        case 'asset_count': {
            const {albumId} = params as AssetCountParams;
            return albumId === null ? demo.assets.length : demo.albumAssets(albumId).length;
        }

        case 'asset_edge': {
            const {albumId, after, depth} = params as AssetEdgeParams;
            const ordered = orderedAssets({albumId, from: null, to: null});
            const start = after
                ? ordered.findIndex((item) => item.takenAt === after.takenAt && item.id === after.id) + 1
                : 0;
            const edge = ordered[start + depth - 1];
            return edge ? cursorOf(edge) : null;
        }

        case 'album_list':
            return demo.albums;

        case 'album_get': {
            const {albumId} = params as AlbumParams;
            return demo.albums.find((item) => item.id === albumId) ?? null;
        }

        case 'album_add': {
            const {uri} = params as AddAlbumParams;
            return demo.buildAlbum(uri);
        }

        case 'album_remove':
            return undefined;

        case 'album_refresh': {
            const {albumId} = params as AlbumParams;
            return demo.album(albumId);
        }

        case 'set_album_cover': {
            const {albumId, assetId} = params as SetAlbumCoverParams;
            const album = demo.album(albumId);
            const rest = album.coverAssetIds.filter((id) => id !== assetId);
            return {...album, coverAssetIds: [assetId, ...rest].slice(0, 4)};
        }

        case 'set_album_description': {
            const {albumId, description} = params as SetAlbumDescriptionParams;
            return {...demo.album(albumId), description};
        }

        case 'asset_detail': {
            const {assetId} = params as {assetId: string};
            return demo.assetDetail(assetId);
        }

        case 'media_source': {
            const source = params as MediaSourceParams;
            if (source.kind === 'localThumb') return demo.localThumb(source.uri);
            const media = demo.media(source.assetId);
            if (source.kind === 'thumb') return media.thumb;
            if (source.kind === 'video') {
                if (!media.video) throw new Error(`demo: ${source.assetId} has no video`);
                return media.video;
            }
            return media.original;
        }

        case 'search_assets': {
            const {query, folderId, from, to, favorite, mediaType, cursor, limit} = params as SearchParams;
            const needle = query.trim().toLowerCase();
            const items = demo.assets.filter(
                (item) =>
                    (!needle || item.name.toLowerCase().includes(needle)) &&
                    (folderId === null || item.folderId === folderId) &&
                    (from === null || item.takenAt >= from) &&
                    (to === null || item.takenAt <= to) &&
                    (favorite === null || item.favorite === favorite) &&
                    (mediaType === null || item.mediaType === mediaType),
            );
            return paged(items, cursor, limit);
        }

        case 'set_favorite': {
            const {assetId, value} = params as SetFavoriteParams;
            return {...demo.asset(assetId), favorite: value};
        }

        case 'move_assets_to_album':
            return [];

        case 'export_assets': {
            const {assetIds} = params as ExportAssetsParams;
            return {exported: assetIds.length, directory: '/storage/emulated/0/Pictures/Source'};
        }

        case 'media_permission_state':
        case 'request_media_permission':
            return {granted: 'full', canRequest: true};

        case 'local_browse': {
            // Android hands the picker a flat bucket list instead of a tree.
            const page: LocalBrowsePage = {current: null, parent: null, entries: demo.localEntries};
            return page;
        }

        case 'local_album_list':
            return demo.localAlbums;

        case 'local_album_add': {
            const {uri, name, albumId} = params as AddLocalAlbumParams;
            return demo.buildLocalAlbum(uri, name, albumId);
        }

        case 'local_album_remove':
            return undefined;

        case 'local_asset_page': {
            const {albumUri, cursor, limit} = params as LocalAssetPageParams;
            return pagedLocal(demo.localAssets(albumUri), cursor, limit);
        }

        case 'enqueue_uploads':
            return demo.uploads;

        case 'upload_tasks':
            return demo.uploads;

        case 'cancel_upload':
            return undefined;

        case 'retry_upload':
            return demo.uploads[0]!;

        case 'subscription_connect':
            return undefined;

        case 'subscription_send': {
            // The page addresses a view; the demo answers it from fixtures.
            const addressed = (params as {message: ClientMessage<null | AssetsQuery> & {view: string}}).message;
            if (addressed.type !== 'subscribe') return undefined;
            const snapshot: ServerMessage<null | AssetsQuery, AlbumMetadata | AssetSummary> = {
                type: 'subscribeApplied',
                requestId: addressed.requestId,
                querySetId: addressed.querySetId,
                queries: addressed.queries.map((query) => ({
                    query,
                    rows: query === null ? demo.albums : orderedAssets(query),
                })),
            };
            for (const subscriber of subscribers.get(addressed.view) ?? []) subscriber(snapshot);
            return undefined;
        }

        default:
            throw new Error(`demo: ${command}(${Object.keys(params).join(', ')}) is not mocked`);
    }
}

export const transport: Transport = {
    async invoke<T>(command: string, params?: Record<string, unknown>): Promise<T> {
        return run(command, params ?? {}) as T;
    },

    /** A subscription is answered with its snapshot; other events never fire. */
    async listen<T>(event: string, handler: (payload: T) => void): Promise<() => void> {
        const lookup = /^subscription:\/\/(\w+)$/.exec(event);
        if (!lookup) return () => {};
        const view = lookup[1]!;
        const subscriber = handler as (payload: unknown) => void;
        const listeners = subscribers.get(view) ?? new Set();
        listeners.add(subscriber);
        subscribers.set(view, listeners);
        return () => listeners.delete(subscriber);
    },
};
