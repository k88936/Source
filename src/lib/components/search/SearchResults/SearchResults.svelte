<script lang="ts">
    import type {AssetSummary} from '#lib/api/bindings/AssetSummary';
    import type {Search} from '#lib/components/search/search.svelte';
    import type {Selection} from '#lib/components/selection/selection.svelte';
    import {i18n} from '#lib/i18n/i18n.svelte';
    import {common} from '#lib/i18n/common.i18n';
    import {search as text} from '#lib/components/search/search.i18n';
    import {app} from '#lib/app/app.svelte';
    import {Search as SearchIcon, WifiOff} from '@lucide/svelte';
    import AssetGrid from '../../assets/AssetGrid/AssetGrid.svelte';
    import EmptyState from '../../assets/EmptyState/EmptyState.svelte';
    import LoadMoreSentinel from '../../assets/LoadMoreSentinel/LoadMoreSentinel.svelte';

    let {
        search,
        selection,
        scoped = false,
        onOpen,
    }: {
        search: Search;
        selection: Selection;
        /** Confined to one album, so the texts can say which library is searched. */
        scoped?: boolean;
        onOpen: (asset: AssetSummary) => void;
    } = $props();

    const t = $derived(i18n.pick(text));
    const c = $derived(i18n.pick(common));
</script>

{#if search.timeline.error}
    <EmptyState
        icon={WifiOff}
        title={t.failed}
        description={search.timeline.error}
        actionLabel={c.retry}
        onAction={() => void search.run()}
    />
{:else if search.timeline.loading && !search.started}
    <div class="flex justify-center py-20">
        <div class="size-7 animate-spin rounded-full border-2 border-line border-t-primary"></div>
    </div>
{:else if !search.started}
    <EmptyState
        icon={SearchIcon}
        title={scoped ? t.idleTitleInAlbum : t.idleTitle}
        description={scoped ? t.idleHintInAlbum : ''}
    />
    {#if search.recent.length > 0}
        <div class="mx-auto max-w-3xl px-4">
            <p class="mb-2 px-1 text-xs font-medium text-muted">{t.recent}</p>
            <div class="flex flex-wrap gap-2">
                {#each search.recent as term (term)}
                    <button
                        onclick={() => search.applyRecent(term)}
                        class="rounded-full bg-gray px-3 py-1.5 text-xs text-muted transition-colors hover:text-dark"
                    >
                        {term}
                    </button>
                {/each}
            </div>
        </div>
    {/if}
{:else if search.timeline.items.length === 0}
    <EmptyState
        icon={SearchIcon}
        title={scoped ? t.noMatchInAlbum : t.noMatch}
        actionLabel={t.clearFilters}
        onAction={() => search.clearFilters()}
    />
{:else}
    <div class="mt-3 flex items-center justify-between px-4 text-xs text-muted">
        <span>{t.results(search.timeline.total)}</span>
        {#if search.timeline.loading}<span>{t.searching}</span>{/if}
    </div>
    <AssetGrid
        items={search.timeline.items}
        columns={app.columns}
        selectionMode={selection.active}
        selected={selection.ids}
        {onOpen}
        onToggle={(asset) => selection.toggle(asset.id)}
        onColumns={(columns) => app.setColumns(columns)}
    />
    <LoadMoreSentinel hasMore={search.timeline.hasMore} onLoad={() => void search.timeline.loadMore()} />
{/if}
