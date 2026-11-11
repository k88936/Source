<script lang="ts">
    import {CircleCheck} from '@lucide/svelte';

    import MediaMosaic from '#lib/components/assets/MediaMosaic/MediaMosaic.svelte';
    import {folderMosaic} from '#lib/components/assets/MediaMosaic/mosaic';
    import {common} from '#lib/i18n/common.i18n';
    import {i18n} from '#lib/i18n/i18n.svelte';
    import type {Mapping, Tone} from '../../sync.svelte';

    let {mapping, onOpen}: {mapping: Mapping; onOpen: () => void} = $props();

    const c = $derived(i18n.pick(common));

    /// Only bytes on the wire animate, so the dot next to a folder is either
    /// moving or still; a scan has no colour of its own.
    function dotClass(tone: Tone): string {
        return tone === 'upload' ? 'bg-primary animate-pulse' : 'bg-line';
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
    </div>
    <div class="min-w-0 flex-1">
        <p class="truncate text-[13px] font-medium leading-tight">{mapping.local.name}</p>
        <p class="truncate text-[9px] leading-tight text-muted">{folderPath(mapping.local.uri)}</p>
        <p class="mt-0.5 truncate text-[10px] leading-tight text-muted">
            {#if mapping.tone === 'upload'}
                {c.uploading} {Math.round(mapping.tally.percent * 100)}%
            {:else}
                {c.items(mapping.local.assetCount)}
            {/if}
        </p>
    </div>
    {#if mapping.finished}
        <CircleCheck size={15} class="shrink-0 text-success-500" />
    {:else}
        <span class="size-1.5 shrink-0 rounded-full {dotClass(mapping.tone)}"></span>
    {/if}
</button>
