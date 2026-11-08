<script lang="ts">
    import type {AssetDetail} from '#lib/api/bindings/AssetDetail';
    import type {AssetSummary} from '#lib/api/bindings/AssetSummary';
    import {api} from '#lib/api/client';
    import AssetInfoSheet from '#lib/components/AssetInfoSheet.svelte';
    import Viewer from '#lib/components/Viewer.svelte';
    import type {ViewerItem} from '#lib/components/viewer-item';
    import {common} from '#lib/i18n/common.i18n';
    import {app} from '#lib/state/app.svelte';
    import {i18n} from '#lib/state/i18n.svelte';
    import type {Timeline} from '#lib/state/timeline.svelte';
    import {Download, Heart, Info} from '@lucide/svelte';
    import {viewer} from './Viewer.i18n';

    let {
        timeline,
        index = $bindable(0),
        onClose,
    }: {
        timeline: Timeline;
        index?: number;
        onClose: () => void;
    } = $props();

    const t = $derived(i18n.pick(viewer));
    const c = $derived(i18n.pick(common));

    let detail = $state<AssetDetail | null>(null);
    let showInfo = $state(false);
    let favoriteBusy = $state(false);

    // The mapping keeps the same object for an unchanged asset, so paging in
    // more items does not restart the media that is already on screen.
    const cached = new Map<string, ViewerItem>();

    function viewerItem(asset: AssetSummary): ViewerItem {
        const existing = cached.get(asset.id);
        if (existing) return existing;
        const item = {
            id: asset.id,
            name: asset.name,
            video: asset.mediaType === 'video',
            thumb: {kind: 'thumb' as const, assetId: asset.id},
            full: {kind: asset.mediaType === 'video' ? ('video' as const) : ('original' as const), assetId: asset.id},
        };
        cached.set(asset.id, item);
        return item;
    }

    let current = $derived(timeline.items[index]);
    let items = $derived(timeline.items.map(viewerItem));

    $effect(() => {
        const asset = current;
        if (!asset) return;
        detail = null;
        showInfo = false;
        let live = true;
        void api
            .assetDetail({assetId: asset.id})
            .then((value) => {
                if (live) detail = value;
            })
            .catch(() => {});
        return () => {
            live = false;
        };
    });

    async function toggleFavorite() {
        const asset = current;
        if (!asset || favoriteBusy) return;
        favoriteBusy = true;
        try {
            const updated = await api.setFavorite({
                assetId: asset.id,
                value: !asset.favorite,
            });
            timeline.items[index] = updated;
            if (detail) detail = {...detail, asset: updated};
        } catch (error) {
            app.showToast(app.describeError(error), 'error');
        } finally {
            favoriteBusy = false;
        }
    }

    async function download() {
        const asset = current;
        if (!asset) return;
        try {
            const result = await api.exportAssets({assetIds: [asset.id]});
            app.showToast(t.exported(result.exported, result.directory), 'success');
        } catch (error) {
            app.showToast(app.describeError(error), 'error');
        }
    }
</script>

<Viewer
    {items}
    total={timeline.total}
    onLoadMore={() => void timeline.loadMore()}
    bind:index
    {onClose}
    onFavorite={() => void toggleFavorite()}
>
    {#snippet actions()}
        <button
            onclick={toggleFavorite}
            class="grid size-11 place-items-center rounded-full hover:bg-white/10"
            aria-label={t.favorite}
        >
            <Heart
                size={23}
                fill={current?.favorite ? 'currentColor' : 'none'}
                class={current?.favorite ? 'text-rose-500' : ''}
            />
        </button>
        <button
            onclick={download}
            class="grid size-11 place-items-center rounded-full hover:bg-white/10"
            aria-label={c.download}
        >
            <Download size={22} />
        </button>
        <button
            onclick={() => (showInfo = !showInfo)}
            class="grid size-11 place-items-center rounded-full hover:bg-white/10"
            aria-label={t.info}
        >
            <Info size={22} />
        </button>
    {/snippet}
    {#snippet overlay()}
        <AssetInfoSheet bind:open={showInfo} asset={current} {detail} />
    {/snippet}
</Viewer>
