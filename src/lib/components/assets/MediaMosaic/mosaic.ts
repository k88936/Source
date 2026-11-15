import type {MediaSourceParams} from '#lib/api/bindings/MediaSourceParams';

/**
 * A lone image fills the frame, and an odd count gives the first image the
 * whole left column: the group still reads as a grid at any length.
 *
 * Shared by every card that shows an album or a folder, so that the same
 * covers look the same wherever they appear.
 */
export function tileClass(count: number, index: number): string {
    if (count === 1) return 'col-span-2';
    return count % 2 === 1 && index === 0 ? 'row-span-2' : '';
}

/** The album's covers, as the mosaic takes them. */
export function albumMosaic(album: {coverAssetIds: string[]}): MediaSourceParams[] {
    return album.coverAssetIds.map((assetId) => ({kind: 'thumb', assetId}));
}

/** The folder's covers, as the mosaic takes them. */
export function folderMosaic(uris: string[]): MediaSourceParams[] {
    return uris.map((uri) => ({kind: 'localThumb', uri}));
}
