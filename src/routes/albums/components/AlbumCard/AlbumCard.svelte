<script lang="ts">
    import type {AlbumMetadata} from '#lib/api/bindings/AlbumMetadata';
    import MediaMosaic from '#lib/components/assets/MediaMosaic/MediaMosaic.svelte';
    import {albumMosaic} from '#lib/components/assets/MediaMosaic/mosaic';
    import {i18n} from '#lib/i18n/i18n.svelte';
    import {card} from './AlbumCard.i18n';

    let {album, onOpen}: {album: AlbumMetadata; onOpen: () => void} = $props();

    const t = $derived(i18n.pick(card));
    const covers = $derived(albumMosaic(album));
</script>

<button
    type="button"
    onclick={onOpen}
    class="group block w-full rounded-2xl border border-transparent p-4 text-left transition-colors hover:border-line hover:bg-gray"
>
    <div class="relative aspect-square overflow-hidden rounded-xl bg-gray">
        <MediaMosaic sources={covers} />
    </div>
    <div class="pt-4">
        <p class="line-clamp-2 text-base font-semibold transition-colors group-hover:text-primary">
            {album.name}
        </p>
        <p class="mt-1 truncate text-xs text-muted">{t.items(album.assetCount, album.path)}</p>
    </div>
</button>
