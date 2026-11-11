<script lang="ts">
    import MediaMosaic from '#lib/components/assets/MediaMosaic/MediaMosaic.svelte';
    import {albumMosaic} from '#lib/components/assets/MediaMosaic/mosaic';
    import {common} from '#lib/i18n/common.i18n';
    import {i18n} from '#lib/i18n/i18n.svelte';
    import type {Group} from '../../sync.svelte';

    let {group, onOpen}: {group: Group; onOpen: () => void} = $props();

    const c = $derived(i18n.pick(common));
    const covers = $derived(albumMosaic(group.album));

    /// The port is where every pipe of this row lands. It is lit while the
    /// album is taking bytes, half lit while it still has rows to work through.
    function portClass(row: Group): string {
        if (row.tally.uploading > 0) return 'bg-primary ring-primary/20';
        if (row.tasks.length > 0) return 'bg-primary/60 ring-primary/10';
        return 'bg-line ring-transparent';
    }
</script>

<button type="button" data-sink={group.album.id} onclick={onOpen} class="group relative block w-full text-left">
    <span
        class="absolute -left-0.75 top-1/2 z-20 size-2 -translate-y-1/2 rounded-full ring-4 ring-light {portClass(
            group,
        )}"
    ></span>
    <div
        class="overflow-hidden rounded-2xl border border-line/60 bg-light/80 transition-all group-hover:border-primary/40 group-active:scale-[0.99]"
    >
        <div class="relative aspect-[4/3] overflow-hidden bg-gray">
            <MediaMosaic sources={covers} iconSize={16} />
        </div>
        <div class="p-2.5">
            <p class="truncate text-[13px] font-semibold leading-tight">{group.album.name}</p>
            <p class="truncate text-[9px] leading-tight text-muted">{group.album.path}</p>
            <p class="mt-1 truncate text-[10px] leading-tight text-muted">
                {c.items(group.album.assetCount)}
            </p>
        </div>
    </div>
</button>
