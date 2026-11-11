import type {AssetSummary} from '#lib/api/bindings/AssetSummary';
import type {LocalAsset} from '#lib/api/bindings/LocalAsset';
import type {MediaSourceParams} from '#lib/api/bindings/MediaSourceParams';

export type ViewerItem = {
    id: string;
    name: string;
    video: boolean;
    thumb: MediaSourceParams;
    full: MediaSourceParams;
};

/**
 * Mapping a list to viewer items. The cache keeps the same object for an
 * unchanged asset, so paging in more items does not restart the media that is
 * already on screen.
 */
function cachedBy<Asset extends {id: string}>(map: (asset: Asset) => ViewerItem) {
    const cache = new Map<string, ViewerItem>();
    return (asset: Asset): ViewerItem => {
        const current = cache.get(asset.id);
        if (current) return current;
        const item = map(asset);
        cache.set(asset.id, item);
        return item;
    };
}

/** A server asset addresses a thumb and one rendition of the original. */
export const timelineItem = cachedBy<AssetSummary>((asset) => ({
    id: asset.id,
    name: asset.name,
    video: asset.mediaType === 'video',
    thumb: {kind: 'thumb', assetId: asset.id},
    full: {kind: asset.mediaType === 'video' ? 'video' : 'original', assetId: asset.id},
}));

/** A local tile addresses the same file for the thumb and the full view. */
export const localItem = cachedBy<LocalAsset>((asset) => ({
    id: asset.id,
    name: asset.name,
    video: asset.mime?.startsWith('video') ?? false,
    thumb: {kind: 'localThumb', uri: asset.uri},
    full: {kind: 'localThumb', uri: asset.uri},
}));
