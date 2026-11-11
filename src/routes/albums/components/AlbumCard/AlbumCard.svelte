<script lang="ts">
    import type {AlbumSummary} from '#lib/api/bindings/AlbumSummary';
    import {Images} from '@lucide/svelte';
    import MediaImage from '#lib/components/assets/MediaImage/MediaImage.svelte';
    import {i18n} from '#lib/i18n/i18n.svelte';
    import {card} from './AlbumCard.i18n';

    let {album, onOpen}: {album: AlbumSummary; onOpen: () => void} = $props();

    const t = $derived(i18n.pick(card));

    let covers = $derived(album.coverAssetIds.slice(0, 4));

    function coverClass(index: number): string {
        if (covers.length === 1) return 'col-span-2 row-span-2';
        if (covers.length === 2 || (covers.length === 3 && index === 0)) return 'row-span-2';
        return '';
    }
</script>

<button
    type="button"
    onclick={onOpen}
    class="group block w-full rounded-2xl border border-transparent p-4 text-left transition-colors hover:border-line hover:bg-gray"
>
    <div class="relative aspect-square overflow-hidden rounded-xl bg-gray">
        {#if covers.length > 0}
            <!-- 2x2 mosaic, like the reference app: one image reads as a single
			     photo, four read as a collection. -->
            <div class="grid size-full grid-cols-2 grid-rows-2 gap-[2px]">
                {#each covers as cover, index (cover)}
                    <MediaImage
                        source={{kind: 'thumb', assetId: cover}}
                        alt=""
                        loading="lazy"
                        decoding="async"
                        class="size-full object-cover {coverClass(index)}"
                    />
                {/each}
            </div>
        {:else}
            <div class="grid size-full place-items-center text-muted">
                <Images size={40} />
            </div>
        {/if}
    </div>
    <div class="pt-4">
        <p class="line-clamp-2 text-base font-semibold transition-colors group-hover:text-primary">
            {album.name}
        </p>
        <p class="mt-1 truncate text-xs text-muted">{t.items(album.assetCount, album.path)}</p>
    </div>
</button>
