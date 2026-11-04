<script lang="ts">
    import type {Snippet} from 'svelte';
    import Icon from './Icon.svelte';

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

    function goBack() {
        if (history.length > 1) history.back();
        else location.href = '/photos';
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
                aria-label="返回"
            >
                <Icon name="back" size={22} />
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
