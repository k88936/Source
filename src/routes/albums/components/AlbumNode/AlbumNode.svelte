<script lang="ts">
    import MediaMosaic from '#lib/components/assets/MediaMosaic/MediaMosaic.svelte';
    import {albumMosaic} from '#lib/components/assets/MediaMosaic/mosaic';
    import {common} from '#lib/i18n/common.i18n';
    import {i18n} from '#lib/i18n/i18n.svelte';
    import type {Group} from '../../albums.svelte.js';

    /// The album as a destination: its cover and its card. What can be done to
    /// the album belongs to its sheet, where the row is read in full.
    let {group, onOpen}: {group: Group; onOpen: () => void} = $props();

    const c = $derived(i18n.pick(common));
    const covers = $derived(albumMosaic(group.album));
    const scanning = $derived(group.tone === 'scan');
    const waiting = $derived(group.job?.state === 'queued');

    /// The port is where every pipe of this row lands. It is lit while the
    /// album is taking bytes, dimmer while it is walking its listing.
    function portClass(tone: Group['tone']): string {
        if (tone === 'upload') return 'bg-primary ring-primary/20';
        if (tone === 'scan') return 'bg-primary/60 ring-primary/10';
        return 'bg-line ring-transparent';
    }
</script>

<button type="button" data-sink={group.album.id} onclick={onOpen} class="group relative block w-full text-left">
    <span
        class="absolute -left-0.75 top-1/2 z-20 size-2 -translate-y-1/2 rounded-full ring-4 ring-light {portClass(
            group.tone,
        )}"
    ></span>
    <div
        class="overflow-hidden rounded-2xl border border-line/60 bg-light/80 transition-all group-hover:border-primary/40 group-active:scale-[0.99]"
    >
        <div class="relative aspect-[4/3] overflow-hidden bg-gray" class:waiting>
            <MediaMosaic sources={covers} iconSize={16} />
            {#if scanning}
                <span class="scan-sheen" aria-hidden="true"></span>
            {/if}
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

<style>
    /* A pass over the cover is the whole animation: one band of light, so a card
       that is working is obvious before any byte moves. */
    .scan-sheen {
        position: absolute;
        inset: 0;
        background: linear-gradient(100deg, transparent 30%, rgb(255 255 255 / 0.45) 50%, transparent 70%);
        transform: translateX(-100%);
        animation: scan-sheen 1.6s ease-in-out infinite;
    }

    .waiting :global(img),
    .waiting::after {
        opacity: 0.7;
    }

    @keyframes scan-sheen {
        to {
            transform: translateX(100%);
        }
    }

    @media (prefers-reduced-motion: reduce) {
        .scan-sheen {
            animation: none;
        }
    }
</style>
