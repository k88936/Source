<script lang="ts">
    import type {AssetDetail} from '#lib/api/bindings/AssetDetail';
    import {api} from '#lib/api/client';
    import AssetInfoSheet from '#lib/components/viewer/AssetInfoSheet/AssetInfoSheet.svelte';
    import {common} from '#lib/i18n/common.i18n';
    import {app} from '#lib/app/app.svelte';
    import {i18n} from '#lib/i18n/i18n.svelte';
    import type {Timeline} from '#lib/components/assets/timeline.svelte';
    import {Download, Heart, Info} from '@lucide/svelte';
    import {viewer} from '../../../../modals/Viewer/Viewer.i18n';

    /**
     * The timeline's own controls, injected into the viewer's header. The viewer
     * shows what it is given, so favourites, export and the info sheet live here
     * and a local album simply injects nothing.
     */
    let {timeline, index}: {timeline: Timeline; index: number} = $props();

    const t = $derived(i18n.pick(viewer));
    const c = $derived(i18n.pick(common));

    let detail = $state<AssetDetail | null>(null);
    let showInfo = $state(false);
    let favoriteBusy = $state(false);

    let current = $derived(timeline.items[index]);

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

    $effect(() => {
        const onKey = (event: KeyboardEvent) => {
            if (event.key === 'f') void toggleFavorite();
        };
        window.addEventListener('keydown', onKey);
        return () => window.removeEventListener('keydown', onKey);
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

<button
    onclick={toggleFavorite}
    class="grid size-11 place-items-center rounded-full hover:bg-white/10"
    aria-label={t.favorite}
>
    <Heart size={23} fill={current?.favorite ? 'currentColor' : 'none'} class={current?.favorite ? 'text-rose-500' : ''} />
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
<AssetInfoSheet bind:open={showInfo} asset={current} {detail} />
