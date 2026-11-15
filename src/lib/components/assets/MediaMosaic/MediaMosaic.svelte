<script lang="ts">
    import type {MediaSourceParams} from '#lib/api/bindings/MediaSourceParams';
    import {Images} from '@lucide/svelte';

    import MediaImage from '../MediaImage/MediaImage.svelte';
    import {tileClass} from './mosaic';

    /// The covers in a grid, like the reference app: one image reads as a single
    /// photo, many read as a collection. The length is whatever the caller has.
    let {sources, iconSize = 40}: {sources: MediaSourceParams[]; iconSize?: number} = $props();
</script>

{#if sources.length > 0}
    <div class="grid size-full auto-rows-fr grid-cols-2 gap-[2px]">
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
