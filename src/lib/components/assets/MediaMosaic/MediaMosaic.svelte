<script lang="ts">
    import type {MediaSourceParams} from '#lib/api/bindings/MediaSourceParams';
    import {Images} from '@lucide/svelte';

    import MediaImage from '../MediaImage/MediaImage.svelte';
    import {tileClass} from './mosaic';

    /// Up to four covers in a 2x2 frame, like the reference app: one image reads
    /// as a single photo, four read as a collection.
    let {sources, iconSize = 40}: {sources: MediaSourceParams[]; iconSize?: number} = $props();
</script>

{#if sources.length > 0}
    <div class="grid size-full grid-cols-2 grid-rows-2 gap-[2px]">
        {#each sources as source, index (index)}
            <MediaImage
                {source}
                alt=""
                loading="lazy"
                decoding="async"
                class="size-full object-cover {tileClass(sources.length, index)}"
            />
        {/each}
    </div>
{:else}
    <div class="grid size-full place-items-center text-muted">
        <Images size={iconSize} />
    </div>
{/if}
