import type {MediaSourceParams} from '#lib/api/bindings/MediaSourceParams';

/**
 * A lone image fills the frame, a pair splits it in columns, a triple gives the
 * first image the whole left column: the group still reads as a grid.
 *
 * Shared by every card that shows an album or a folder, so that one, two, three
 * and four covers look the same wherever they appear.
 */
export function tileClass(count: number, index: number): string {
    if (count === 1) return 'col-span-2 row-span-2';
    if (count === 2 || (count === 3 && index === 0)) return 'row-span-2';
    return '';
}

/** The album's first four covers, as the mosaic takes them. */
export function albumMosaic(album: {coverAssetIds: string[]}): MediaSourceParams[] {
    return album.coverAssetIds.slice(0, 4).map((assetId) => ({kind: 'thumb', assetId}));
}

/** The folder's first four covers, as the mosaic takes them. */
export function folderMosaic(uris: string[]): MediaSourceParams[] {
    return uris.slice(0, 4).map((uri) => ({kind: 'localThumb', uri}));
}
