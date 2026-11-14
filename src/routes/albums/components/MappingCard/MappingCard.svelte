<script lang="ts">
    import MediaMosaic from '#lib/components/assets/MediaMosaic/MediaMosaic.svelte';
    import {folderMosaic} from '#lib/components/assets/MediaMosaic/mosaic';
    import {common} from '#lib/i18n/common.i18n';
    import {i18n} from '#lib/i18n/i18n.svelte';
    import type {Mapping, Tone} from '../../albums.svelte.js';

    /// One folder as a source: its cover and what it is doing. What can be
    /// done to the folder belongs to its sheet, where the row is read in full.
    let {mapping, onOpen}: {mapping: Mapping; onOpen: () => void} = $props();

    const c = $derived(i18n.pick(common));
    const scanning = $derived(mapping.tone === 'scan');

    /// How far the files this folder is sending have got. Only the rows that are
    /// moving are counted, so this is a progress of the wire, not a backlog.
    const percent = $derived.by(() => {
        let sent = 0;
        let total = 0;
        for (const task of mapping.tasks) {
            sent += task.uploaded;
            total += task.total;
        }
        return total > 0 ? Math.round((sent / total) * 100) : 0;
    });

    /// Only bytes on the wire animate at full strength; a scan pulses dimmer,
    /// and a still folder has a plain dot.
    function dotClass(tone: Tone): string {
        if (tone === 'upload') return 'bg-primary animate-pulse';
        if (tone === 'scan') return 'bg-primary/60';
        return 'bg-line';
    }

    /// The last two segments are enough to tell two folders of the same device
    /// apart, and they fit the card.
    function folderPath(uri: string): string {
        const parts = uri
            .replace(/^content:\/\/media\/external\//, '')
            .split('/')
            .filter(Boolean);
        return parts.slice(-2).join('/');
    }
</script>

<button
    type="button"
    data-edge={mapping.local.id}
    data-album={mapping.local.albumId}
    onclick={onOpen}
    class="group relative flex w-full items-center gap-2.5 rounded-xl border border-line/60 bg-light/80 p-2 text-left transition-all hover:border-line hover:bg-gray/50 active:scale-[0.99]"
>
    <div class="relative size-9 shrink-0 overflow-hidden rounded-lg bg-gray">
        <MediaMosaic sources={folderMosaic(mapping.local.coverUris)} iconSize={16} />
        {#if scanning}
            <span class="scan-sheen" aria-hidden="true"></span>
        {/if}
    </div>
    <div class="min-w-0 flex-1">
        <p class="truncate text-[13px] font-medium leading-tight">{mapping.local.name}</p>
        <p class="truncate text-[9px] leading-tight text-muted">{folderPath(mapping.local.uri)}</p>
        <p class="mt-0.5 truncate text-[10px] leading-tight text-muted">
            {#if mapping.tone === 'upload'}
                {c.uploading} {percent}%
            {:else}
                {c.items(mapping.local.assetCount)}
            {/if}
        </p>
    </div>
    <span class="size-1.5 shrink-0 rounded-full {dotClass(mapping.tone)}"></span>
</button>

<style>
    /* A pass over the folder's cover is the whole animation: a folder that is
       walking its directory is obvious before a single byte is on the wire. */
    .scan-sheen {
        position: absolute;
        inset: 0;
        background: linear-gradient(100deg, transparent 30%, rgb(255 255 255 / 0.45) 50%, transparent 70%);
        transform: translateX(-100%);
        animation: scan-sheen 1.6s ease-in-out infinite;
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
