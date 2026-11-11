<script lang="ts">
    import type {Search} from '#lib/components/search/search.svelte';
    import {i18n} from '#lib/i18n/i18n.svelte';
    import {search as text} from '#lib/components/search/search.i18n';
    import {Search as SearchIcon, X} from '@lucide/svelte';

    let {search, scoped = false}: {search: Search; scoped?: boolean} = $props();

    const t = $derived(i18n.pick(text));
    const placeholder = $derived(scoped ? t.placeholderInAlbum : t.placeholder);

    let field = $state<HTMLInputElement>();

    // The button that opened the mode was a request to type.
    $effect(() => {
        field?.focus();
    });
</script>

<div class="relative w-full">
    <span class="pointer-events-none absolute top-1/2 left-3 -translate-y-1/2 text-muted">
        <SearchIcon size={17} />
    </span>
    <input
        bind:this={field}
        value={search.query}
        oninput={(event) => search.setQuery(event.currentTarget.value)}
        onkeydown={(event) => {
            if (event.key === 'Enter') {
                event.currentTarget.blur();
                search.remember();
            }
        }}
        type="text"
        inputmode="search"
        enterkeyhint="search"
        {placeholder}
        class="field h-10 rounded-full pl-9 {search.query ? 'pr-9' : 'pr-3'}"
        aria-label={placeholder}
    />
    {#if search.query}
        <button
            onclick={() => search.setQuery('')}
            class="absolute top-1/2 right-2 grid size-6 -translate-y-1/2 place-items-center rounded-full bg-light text-muted hover:text-dark"
            aria-label={t.clear}
        >
            <X size={13} />
        </button>
    {/if}
</div>
