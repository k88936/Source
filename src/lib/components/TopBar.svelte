<script lang="ts">
    import type {Snippet} from 'svelte';
    import {ChevronLeft} from '@lucide/svelte';
    import {resolve} from '$app/paths';
    import {app} from '#lib/state/app.svelte';
    import {i18n} from '#lib/state/i18n.svelte';
    import {common} from '#lib/i18n/common.i18n';

    let {
        title,
        subtitle = '',
        back = false,
        transparent = false,
        content,
        actions,
    }: {
        title: string;
        subtitle?: string;
        back?: boolean;
        transparent?: boolean;
        /** Replaces the title and the subtitle, e.g. by a search field. */
        content?: Snippet;
        actions?: Snippet;
    } = $props();

    const t = $derived(i18n.pick(common));

    /** Width of the trailing actions, mirrored on the empty leading side. */
    let actionsWidth = $state(0);

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
    <!-- Edge on a phone marks the same state this way: one thin grey line at
    the top, not a toast, gone the moment the client answers again. -->
    {#if app.setup === 'offline' && app.account}
        <p class="flex h-6 items-center justify-center bg-gray text-[11px] text-muted">
            {t.offline}
        </p>
    {/if}
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
        {#if content}
            <div class="min-w-0 flex-1" style="padding-inline-start: {actionsWidth}px">
                {@render content()}
            </div>
        {:else}
            <div class="flex min-w-0 flex-1 items-baseline gap-2">
                <h1 class="truncate text-base font-semibold sidebar:text-sm sidebar:font-medium">{title}</h1>
                {#if subtitle}
                    <span class="truncate text-xs text-muted">{subtitle}</span>
                {/if}
            </div>
        {/if}
        <div bind:clientWidth={actionsWidth} class="flex items-center gap-1">
            {@render actions?.()}
        </div>
    </div>
</header>
