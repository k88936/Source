<script lang="ts">
    import type {Snippet} from 'svelte';
    import {ChevronLeft} from '@lucide/svelte';
    import {resolve} from '$app/paths';
    import {i18n} from '#lib/state/i18n.svelte';
    import {common} from '#lib/i18n/common.i18n';

    let {
        title,
        subtitle = '',
        back = false,
        transparent = false,
        actions,
    }: {
        title: string;
        subtitle?: string;
        back?: boolean;
        transparent?: boolean;
        actions?: Snippet;
    } = $props();

    const t = $derived(i18n.pick(common));

    function goBack() {
        if (history.length > 1) history.back();
        else location.href = resolve('/photos');
    }
</script>

<header
    class="sticky top-0 z-20 pt-[var(--safe-top)] {transparent
        ? 'bg-transparent'
        : 'border-b border-line bg-light/90 backdrop-blur-xl'}"
>
    <div class="flex h-(--navbar-height) w-full items-center gap-2 px-2 sidebar:px-6">
        {#if back}
            <button
                onclick={goBack}
                class="grid size-9 place-items-center rounded-full text-dark transition-colors hover:bg-gray"
                aria-label={t.back}
            >
                <ChevronLeft size={22} />
            </button>
        {/if}
        <div class="flex min-w-0 flex-1 items-baseline gap-2">
            <h1 class="truncate text-base font-semibold sidebar:text-sm sidebar:font-medium">{title}</h1>
            {#if subtitle}
                <span class="truncate text-xs text-muted">{subtitle}</span>
            {/if}
        </div>
        <div class="flex items-center gap-1">
            {@render actions?.()}
        </div>
    </div>
</header>
