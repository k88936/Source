<script lang="ts">
    import type {ScreenEntry, StackController} from '../navigation.svelte';
    import type {StackOptions} from '../types';

    let {
        controller,
        entry,
        options,
    }: {controller: StackController; entry: ScreenEntry; options: StackOptions} = $props();

    // While a mode takes the back key, the bar must not offer a second way out.
    const canBack = $derived(controller.entries.length > 1 && !controller.root.hasMode(entry));
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
            <button
                onclick={() => controller.root.back(controller)}
                class="grid size-9 place-items-center rounded-full text-dark transition-colors hover:bg-gray"
                aria-label={options.headerBackLabel ?? 'Back'}
            >
                <svg
                    width="22"
                    height="22"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                >
                    <path d="m15 18-6-6 6-6" />
                </svg>
            </button>
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
