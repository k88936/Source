<script lang="ts">
    import type {LocalAsset} from '#lib/api/bindings/LocalAsset';
    import Viewer from '#lib/components/Viewer.svelte';
    import type {ViewerItem} from '#lib/components/viewer-item';

    let {
        items,
        index = $bindable(0),
        total,
        onLoadMore,
        onClose,
    }: {
        items: LocalAsset[];
        index?: number;
        total: number;
        onLoadMore: () => void;
        onClose: () => void;
    } = $props();

    // A local tile addresses the same file for the thumb and the full view;
    // there is no server-side rendition of it.
    const cached = new Map<string, ViewerItem>();

    function viewerItem(asset: LocalAsset): ViewerItem {
        const existing = cached.get(asset.id);
        if (existing) return existing;
        const item = {
            id: asset.id,
            name: asset.name,
            video: asset.mime?.startsWith('video') ?? false,
            thumb: {kind: 'localThumb' as const, uri: asset.uri},
            full: {kind: 'localThumb' as const, uri: asset.uri},
        };
        cached.set(asset.id, item);
        return item;
    }

    let viewerItems = $derived(items.map(viewerItem));
</script>

<Viewer items={viewerItems} bind:index {total} {onLoadMore} {onClose} />
