<script lang="ts">
    import {IconButton} from '@immich/ui';
    import {mdiChevronLeft} from '@mdi/js';
    import type {StackHeaderProps} from 'svelte-navigation';
    import {common} from '#lib/i18n/common.i18n';
    import {i18n} from '#lib/state/i18n.svelte';

    let {options, canBack, back}: StackHeaderProps = $props();

    const c = $derived(i18n.pick(common));
    const transparent = $derived(options.headerTransparent === true);
</script>

<!-- The layer scrolls its own content, so the bar only has to stay above it. -->
<header
    class="z-20 shrink-0 {transparent
        ? 'bg-transparent'
        : 'border-b border-line bg-light/90 backdrop-blur-xl'}"
    style:padding-top="var(--safe-top, env(safe-area-inset-top, 0px))"
>
    {#if options.headerBanner}
        {@render options.headerBanner()}
    {/if}
    <div class="flex w-full items-center gap-2 px-2" style:height="var(--navbar-height, 3.5rem)">
        {#if canBack}
            <IconButton
                icon={mdiChevronLeft}
                variant="ghost"
                size="small"
                aria-label={options.headerBackLabel ?? c.back}
                onclick={back}
            />
        {/if}
        {#if options.headerContent}
            <div class="min-w-0 flex-1">{@render options.headerContent()}</div>
        {:else}
            <div class="flex min-w-0 flex-1 items-baseline gap-2">
                <h1 class="truncate text-base font-semibold">{options.title ?? ''}</h1>
                {#if options.subtitle}
                    <span class="truncate text-xs text-muted">{options.subtitle}</span>
                {/if}
            </div>
        {/if}
        <div class="flex items-center gap-1">
            {#if options.headerActions}
                {@render options.headerActions()}
            {/if}
        </div>
    </div>
</header>
