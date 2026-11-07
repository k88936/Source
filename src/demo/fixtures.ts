import type {AccountState} from '#lib/api/bindings/AccountState';
import type {AlbumSummary} from '#lib/api/bindings/AlbumSummary';
import type {AppInfo} from '#lib/api/bindings/AppInfo';
import type {AssetDetail} from '#lib/api/bindings/AssetDetail';
import type {AssetSummary} from '#lib/api/bindings/AssetSummary';
import type {CacheStats} from '#lib/api/bindings/CacheStats';
import type {Capacity} from '#lib/api/bindings/Capacity';
import type {FolderRef} from '#lib/api/bindings/FolderRef';
import type {Job} from '#lib/api/bindings/Job';
import type {JobKind} from '#lib/api/bindings/JobKind';
import type {LocalAlbumSummary} from '#lib/api/bindings/LocalAlbumSummary';
import type {LocalAsset} from '#lib/api/bindings/LocalAsset';
import type {LocalEntry} from '#lib/api/bindings/LocalEntry';
import type {MediaType} from '#lib/api/bindings/MediaType';
import type {UploadTask} from '#lib/api/bindings/UploadTask';
import bing from './bing.json';

const BING = 'https://www.bing.com';
const DAY = 86_400_000;
const HOUR = 3_600_000;
const THUMB = '&w=400&h=400&c=7';

interface BingImage {
    date: string;
    url: string;
    copyright: string;
}

const images: BingImage[] = bing;

/** `bing.json` dates are `YYYYMMDD`; the hour keeps a day clear of midnight. */
function dayOf(date: string): number {
    return new Date(Number(date.slice(0, 4)), Number(date.slice(4, 6)) - 1, Number(date.slice(6, 8)), 19, 24).getTime();
}

/** `bing.json` is newest first, so every index is also "days before today". */
const newest = dayOf(images[0]!.date);

export const account: AccountState = {
    server: 'https://demo.local',
    siteName: 'Source Demo',
    user: {id: 'user-demo', name: 'Demo', email: 'demo@example.com'},
};

export const appInfo: AppInfo = {name: 'Source', version: '0.1.0', platform: 'android'};

export const capacity: Capacity = {total: 51_539_607_552, used: 12_884_901_888, storagePackTotal: 0};

export const cacheStats: CacheStats = {
    thumbBytes: 48_234_496,
    thumbFiles: 342,
    mediaBytes: 214_748_364,
    mediaFiles: 26,
    budgetBytes: 536_870_912,
};

/**
 * Two public clips, so the viewer's playback, duration badge and the timeline's
 * video filter all have something real to show.
 */
const VIDEOS = new Map<number, {url: string; durationMs: number}>([
    [2, {url: 'https://interactive-examples.mdn.mozilla.net/media/cc0-videos/flower.mp4', durationMs: 5_000}],
    [
        9,
        {
            url: 'https://test-videos.co.uk/vids/bigbuckbunny/mp4/h264/720/Big_Buck_Bunny_720_10s_2MB.mp4',
            durationMs: 10_000,
        },
    ],
]);

/**
 * Photos from a real roll are not all 16:9, and the timeline's justified rows
 * only look right when they are not.
 */
const RATIOS = [3 / 4, 4 / 3, 1, 3 / 2, 2 / 3, 16 / 9];

function buildAsset(index: number): AssetSummary {
    const video = VIDEOS.get(index);
    const mediaType: MediaType = video ? 'video' : 'image';
    const width = video ? 1280 : 1200;
    const height = video ? 720 : Math.round(width / RATIOS[index % RATIOS.length]!);
    return {
        id: `asset-${index + 1}`,
        folderId: 'folder-2026',
        name: video ? `VID_${8002 + index}.mp4` : `IMG_${1042 + index}.jpg`,
        mediaType,
        mime: video ? 'video/mp4' : 'image/jpeg',
        size: video ? 8_412_664 + index * 11_337 : 2_421_884 + index * 9_371,
        takenAt: newest - Math.floor(index / 3) * DAY - (index % 3) * 3 * HOUR,
        takenAtSource: 'exif',
        width,
        height,
        durationMs: video?.durationMs ?? null,
        favorite: index % 8 === 0,
        thumbHash: null,
    };
}

export const assets: AssetSummary[] = Array.from({length: 24}, (_, index) => buildAsset(index));

export interface Media {
    thumb: string;
    original: string;
    video: string | null;
}

function mediaOf(index: number): Media {
    const picture = images[index]!;
    return {
        thumb: `${BING}${picture.url}${THUMB}`,
        original: `${BING}${picture.url}`,
        video: VIDEOS.get(index)?.url ?? null,
    };
}

export function asset(id: string): AssetSummary {
    const found = assets.find((item) => item.id === id);
    if (!found) throw new Error(`demo: unknown asset ${id}`);
    return found;
}

export function media(id: string): Media {
    return mediaOf(Number(id.slice('asset-'.length)) - 1);
}

interface AlbumDef {
    id: string;
    name: string;
    slug: string;
    description: string;
    from: number;
}

const ALBUM_DEFS: AlbumDef[] = [
    {id: 'album-1', name: '山海之间', slug: 'coast', description: '海边的清晨, 山脊的黄昏', from: 0},
    {id: 'album-2', name: '城市夜色', slug: 'city', description: '雨后的霓虹和天台', from: 6},
    {id: 'album-3', name: '北方公路', slug: 'road', description: '一路向北的七天', from: 12},
    {id: 'album-4', name: '夏日碎片', slug: 'summer', description: '没有主题的夏天', from: 18},
];

function itemsOf(def: AlbumDef): AssetSummary[] {
    return assets.slice(def.from, def.from + 6);
}

export const albums: AlbumSummary[] = ALBUM_DEFS.map((def, order) => {
    const items = itemsOf(def);
    return {
        id: def.id,
        name: def.name,
        uri: `cloudreve://my/Photos/${def.slug}`,
        path: `/Photos/${def.slug}`,
        assetCount: items.length,
        coverAssetIds: items.slice(0, 4).map((item) => item.id),
        description: def.description,
        updatedAt: newest - order * DAY,
        sync: {failed: false, error: null, listedAt: newest - order * DAY + HOUR},
    };
});

export function album(id: string): AlbumSummary {
    const found = albums.find((item) => item.id === id);
    if (!found) throw new Error(`demo: unknown album ${id}`);
    return found;
}

export function albumAssets(id: string): AssetSummary[] {
    const def = ALBUM_DEFS.find((item) => item.id === id);
    if (!def) throw new Error(`demo: unknown album ${id}`);
    return itemsOf(def);
}

const albumOf = new Map<string, AlbumSummary>(
    ALBUM_DEFS.flatMap((def) => itemsOf(def).map((item) => [item.id, album(def.id)] as const)),
);

export function albumOfAsset(id: string): AlbumSummary | null {
    return albumOf.get(id) ?? null;
}

export function assetDetail(id: string): AssetDetail {
    const found = asset(id);
    const owner = albumOfAsset(id);
    return {
        asset: found,
        albumName: owner?.name ?? null,
        albumPath: owner?.path ?? null,
        exif: [
            {key: 'make', label: 'Make', value: 'SONY'},
            {key: 'model', label: 'Model', value: 'ILCE-7M4'},
            {key: 'lens', label: 'Lens', value: 'FE 35mm F1.8'},
            {key: 'focalLength', label: 'Focal length', value: '35 mm'},
            {key: 'fNumber', label: 'Aperture', value: 'f/2.8'},
            {key: 'exposureTime', label: 'Shutter', value: '1/250 s'},
            {key: 'iso', label: 'ISO', value: '400'},
        ],
        downloadName: found.name,
    };
}

export function buildAlbum(uri: string): AlbumSummary {
    const name = decodeURIComponent(uri.slice(uri.lastIndexOf('/') + 1));
    return {
        id: `album-${albums.length + 1}`,
        name,
        uri,
        path: `/Photos/${name}`,
        assetCount: 0,
        coverAssetIds: [],
        description: null,
        updatedAt: Date.now(),
        sync: null,
    };
}

const ROOT: FolderRef = {id: 'folder-root', uri: 'cloudreve://my', name: '我的文件', path: '/'};

const FOLDERS: FolderRef[] = [
    ROOT,
    {id: 'folder-photos', uri: 'cloudreve://my/Photos', name: 'Photos', path: '/Photos'},
    {id: 'folder-2026', uri: 'cloudreve://my/Photos/2026', name: '2026', path: '/Photos/2026'},
    {id: 'folder-trips', uri: 'cloudreve://my/Trips', name: 'Trips', path: '/Trips'},
];

export function folder(uri: string): FolderRef {
    const found = FOLDERS.find((item) => item.uri === uri);
    if (!found) throw new Error(`demo: unknown folder ${uri}`);
    return found;
}

export function foldersInside(uri: string): FolderRef[] {
    const prefix = `${folder(uri).uri}/`;
    return FOLDERS.filter((item) => item.uri.startsWith(prefix) && !item.uri.slice(prefix.length).includes('/'));
}

export const localEntries: LocalEntry[] = [
    {uri: 'content://media/external/images/bucket/1001', name: 'Camera', path: 'DCIM/Camera'},
    {uri: 'content://media/external/images/bucket/1002', name: 'Screenshots', path: 'Pictures/Screenshots'},
    {uri: 'content://media/external/images/bucket/1003', name: 'WeiXin', path: 'Pictures/WeiXin'},
];

const localThumbs = new Map<string, string>();

function buildLocalAssets(seed: number, count: number): LocalAsset[] {
    return Array.from({length: count}, (_, index) => {
        const uri = `content://media/external/images/media/${seed}${index + 1}`;
        localThumbs.set(uri, mediaOf(seed * 3 + index).thumb);
        return {
            id: `local-${seed}-${index + 1}`,
            albumId: `local-album-${seed}`,
            name: `IMG_${3000 + seed * 100 + index}.jpg`,
            mime: 'image/jpeg',
            size: 1_884_213 + index * 7_733,
            dateTaken: newest - index * 6 * HOUR,
            uri,
            width: 1920,
            height: 1080,
            durationMs: null,
        };
    });
}

const LOCAL_ASSETS = new Map<string, LocalAsset[]>([
    ['content://media/external/images/bucket/1001', buildLocalAssets(1, 6)],
    ['content://media/external/images/bucket/1002', buildLocalAssets(2, 4)],
]);

export function localAssets(albumUri: string): LocalAsset[] {
    const found = LOCAL_ASSETS.get(albumUri);
    if (!found) throw new Error(`demo: unknown local album ${albumUri}`);
    return found;
}

export function localThumb(uri: string): string {
    const found = localThumbs.get(uri);
    if (!found) throw new Error(`demo: unknown local asset ${uri}`);
    return found;
}

export const localAlbums: LocalAlbumSummary[] = [
    {
        id: 'local-album-1',
        uri: 'content://media/external/images/bucket/1001',
        name: 'Camera',
        albumId: 'album-1',
        albumName: album('album-1').name,
        assetCount: 6,
    },
    {
        id: 'local-album-2',
        uri: 'content://media/external/images/bucket/1002',
        name: 'Screenshots',
        albumId: 'album-2',
        albumName: album('album-2').name,
        assetCount: 4,
    },
];

export function buildLocalAlbum(uri: string, name: string, albumId: string): LocalAlbumSummary {
    return {
        id: `local-album-${localAlbums.length + 1}`,
        uri,
        name,
        albumId,
        albumName: album(albumId).name,
        assetCount: 0,
    };
}

export const uploads: UploadTask[] = [
    {
        id: 'upload-1',
        localAssetId: 'local-1-1',
        name: 'IMG_3100.jpg',
        destUri: 'cloudreve://my/Photos/coast',
        destName: null,
        state: 'uploading',
        uploaded: 1_842_000,
        total: 3_421_884,
        speed: 620_000,
        etaSeconds: 3,
        attempts: 1,
        error: null,
    },
    {
        id: 'upload-2',
        localAssetId: 'local-1-2',
        name: 'IMG_3101.jpg',
        destUri: 'cloudreve://my/Photos/coast',
        destName: null,
        state: 'queued',
        uploaded: 0,
        total: 2_884_120,
        speed: 0,
        etaSeconds: null,
        attempts: 1,
        error: null,
    },
];

export const jobs: Job<JobKind>[] = [
    {
        id: 'album-1',
        kind: 'index',
        state: 'done',
        progress: {done: 24, total: 24},
        submittedAt: newest - 2 * HOUR,
        startedAt: newest - 2 * HOUR,
        finishedAt: newest - 2 * HOUR + 4_000,
    },
    {
        id: 'local-album-1',
        kind: 'sync',
        state: 'running',
        progress: {done: 2, total: null},
        submittedAt: newest - HOUR,
        startedAt: newest - HOUR,
        finishedAt: null,
    },
    {
        id: 'upload-1',
        kind: 'upload',
        state: 'running',
        progress: {done: 1_842_000, total: 3_421_884},
        submittedAt: newest - HOUR / 2,
        startedAt: newest - HOUR / 2,
        finishedAt: null,
    },
];
