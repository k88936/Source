<script lang="ts">
    import {onMount} from 'svelte';
    import {api} from '#lib/api/client';
    import type {AlbumSummary} from '#lib/api/bindings/AlbumSummary';
    import type {AssetSummary} from '#lib/api/bindings/AssetSummary';
    import type {MediaType} from '#lib/api/bindings/MediaType';
    import AlbumPickerSheet from '#lib/components/AlbumPickerSheet.svelte';
    import AssetGrid from '#lib/components/AssetGrid.svelte';
    import EmptyState from '#lib/components/EmptyState.svelte';
    import LoadMoreSentinel from '#lib/components/LoadMoreSentinel.svelte';
    import {Heart, Image, Search, SquareDashed, Video, WifiOff, X} from '@lucide/svelte';
    import SelectAllButton from '#lib/components/SelectAllButton.svelte';
    import SelectionBar from '#lib/components/SelectionBar.svelte';
    import TimelineViewer from '#lib/components/TimelineViewer.svelte';
    import TopBar from '#lib/components/TopBar.svelte';
    import {common} from '#lib/i18n/common.i18n';
    import {app} from '#lib/state/app.svelte';
    import {changes} from '#lib/state/changes';
    import {i18n} from '#lib/state/i18n.svelte';
    import {Selection} from '#lib/state/selection.svelte';
    import {Timeline} from '#lib/state/timeline.svelte';
    import {search} from './page.i18n';

    const RECENT_KEY = 'recent-searches';
    const RECENT_LIMIT = 8;

    let query = $state('');
    let mediaType = $state<MediaType | null>(null);
    let favorite = $state(false);
    let from = $state('');
    let to = $state('');
    let searched = $state(false);
    let recent = $state<string[]>([]);
    let viewerOpen = $state(false);
    let viewerIndex = $state(0);
    let pickerOpen = $state(false);
    let moving = $state(false);

    const t = $derived(i18n.pick(search));
    const c = $derived(i18n.pick(common));

    // Replaced by `run()` before the first request; the empty query never loads.
    const timeline = new Timeline((cursor, limit) =>
        api.searchAssets({
            query: '',
            from: null,
            to: null,
            favorite: null,
            mediaType: null,
            cursor,
            limit,
        }),
    );
    const selection = new Selection();

    function dateFrom(value: string): number | null {
        return value ? new Date(`${value}T00:00:00`).getTime() : null;
    }

    function dateTo(value: string): number | null {
        return value ? new Date(`${value}T23:59:59.999`).getTime() : null;
    }

    function hasFilters(): boolean {
        return Boolean(query.trim() || mediaType || favorite || from || to);
    }

    $effect(() => {
        // Touch every filter so the effect re-runs on change.
        const snapshot = {query, mediaType, favorite, from, to};
        const timer = setTimeout(() => {
            void run(snapshot);
        }, 280);
        return () => clearTimeout(timer);
    });

    $effect(() => {
        return changes.subscribe((change) => {
            if (change.kind === 'albums' && searched) void run({query, mediaType, favorite, from, to});
        });
    });

    async function run(snapshot: {
        query: string;
        mediaType: MediaType | null;
        favorite: boolean;
        from: string;
        to: string;
    }) {
        const active = Boolean(
            snapshot.query.trim() || snapshot.mediaType || snapshot.favorite || snapshot.from || snapshot.to,
        );
        if (!active) {
            timeline.reset();
            searched = false;
            return;
        }
        timeline.setLoader((cursor, limit) =>
            api.searchAssets({
                query: snapshot.query.trim(),
                from: dateFrom(snapshot.from),
                to: dateTo(snapshot.to),
                favorite: snapshot.favorite ? true : null,
                mediaType: snapshot.mediaType,
                cursor,
                limit,
            }),
        );
        await timeline.reload();
        searched = true;
        remember(snapshot.query);
    }

    function loadRecent(): void {
        if (typeof localStorage === 'undefined') return;
        try {
            const raw = localStorage.getItem(RECENT_KEY);
            const parsed = raw ? JSON.parse(raw) : [];
            if (Array.isArray(parsed)) recent = parsed.filter((item) => typeof item === 'string');
        } catch {
            recent = [];
        }
    }

    function remember(value: string): void {
        const term = value.trim();
        if (!term) return;
        recent = [term, ...recent.filter((item) => item !== term)].slice(0, RECENT_LIMIT);
        if (typeof localStorage !== 'undefined') {
            localStorage.setItem(RECENT_KEY, JSON.stringify(recent));
        }
    }

    function applyRecent(term: string): void {
        query = term;
    }

    function clearFilters(): void {
        query = '';
        mediaType = null;
        favorite = false;
        from = '';
        to = '';
    }

    function open(asset: AssetSummary) {
        if (selection.active) {
            selection.toggle(asset.id);
            return;
        }
        viewerIndex = timeline.items.findIndex((item) => item.id === asset.id);
        if (viewerIndex >= 0) viewerOpen = true;
    }

    async function downloadSelected() {
        if (!selection.count) return;
        try {
            const result = await api.exportAssets({assetIds: selection.ids});
            app.showToast(t.exported(result.exported), 'success');
            selection.exit();
        } catch (err) {
            app.showToast(app.describeError(err), 'error');
        }
    }

    async function moveToAlbum(album: AlbumSummary) {
        moving = true;
        try {
            const moved = await api.moveAssetsToAlbum({
                assetIds: [...selection.ids],
                albumId: album.id,
            });
            for (const asset of moved) timeline.upsert(asset);
            app.showToast(t.moved(moved.length, album.name), 'success');
            selection.exit();
        } catch (err) {
            app.showToast(app.describeError(err), 'error');
        } finally {
            moving = false;
        }
    }

    onMount(loadRecent);
</script>

<svelte:head><title>{t.title} - Source</title></svelte:head>

<TopBar title={selection.active ? t.selected(selection.count) : t.title}>
    {#snippet actions()}
        {#if selection.active}
            <SelectAllButton {selection} ids={timeline.items.map((item) => item.id)} />
            <button
                onclick={() => selection.exit()}
                class="grid size-9 place-items-center rounded-full text-dark hover:bg-gray"
                aria-label={c.cancel}
            >
                <X size={21} />
            </button>
        {:else if timeline.items.length > 0}
            <button
                onclick={() => selection.start()}
                class="grid size-9 place-items-center rounded-full text-dark hover:bg-gray"
                aria-label={c.select}
            >
                <SquareDashed size={20} />
            </button>
        {/if}
    {/snippet}
</TopBar>

<div class="mx-auto max-w-3xl px-4 pt-2">
    <div class="relative">
        <span class="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-muted">
            <Search size={19} />
        </span>
        <input bind:value={query} type="search" placeholder={t.placeholder} class="field h-11 pl-10 pr-10" />
        {#if query}
            <button
                onclick={() => (query = '')}
                class="absolute right-2 top-1/2 grid size-7 -translate-y-1/2 place-items-center rounded-full bg-light text-muted hover:text-dark"
                aria-label={t.clear}
            >
                <X size={14} />
            </button>
        {/if}
    </div>

    <div class="mt-3 flex gap-2 overflow-x-auto pb-1">
        <button
            onclick={() => (mediaType = null)}
            class="shrink-0 rounded-full px-3.5 py-1.5 text-xs font-medium transition-colors {mediaType === null
                ? 'bg-primary text-on-primary'
                : 'bg-gray text-muted hover:text-dark'}"
        >
            {c.all}
        </button>
        <button
            onclick={() => (mediaType = 'image')}
            class="shrink-0 rounded-full px-3.5 py-1.5 text-xs font-medium transition-colors {mediaType === 'image'
                ? 'bg-primary text-on-primary'
                : 'bg-gray text-muted hover:text-dark'}"
        >
            <Image size={13} class="mr-1 inline" />{t.image}
        </button>
        <button
            onclick={() => (mediaType = 'video')}
            class="shrink-0 rounded-full px-3.5 py-1.5 text-xs font-medium transition-colors {mediaType === 'video'
                ? 'bg-primary text-on-primary'
                : 'bg-gray text-muted hover:text-dark'}"
        >
            <Video size={13} class="mr-1 inline" />{t.video}
        </button>
        <button
            onclick={() => (favorite = !favorite)}
            class="shrink-0 rounded-full px-3.5 py-1.5 text-xs font-medium transition-colors {favorite
                ? 'bg-primary text-on-primary'
                : 'bg-gray text-muted hover:text-dark'}"
        >
            <Heart size={13} class="mr-1 inline" />{t.favorites}
        </button>
    </div>

    <div class="mt-2 flex items-center gap-2">
        <label class="flex flex-1 items-center gap-1.5 rounded-lg bg-gray px-3 py-2 text-xs text-muted">
            <span>{t.from}</span>
            <input bind:value={from} type="date" class="w-full bg-transparent text-xs text-dark outline-none" />
        </label>
        <label class="flex flex-1 items-center gap-1.5 rounded-lg bg-gray px-3 py-2 text-xs text-muted">
            <span>{t.to}</span>
            <input bind:value={to} type="date" class="w-full bg-transparent text-xs text-dark outline-none" />
        </label>
        {#if hasFilters()}
            <button onclick={clearFilters} class="shrink-0 px-2 text-xs font-medium text-primary">
                {t.clear}
            </button>
        {/if}
    </div>
</div>

{#if timeline.error}
    <EmptyState
        icon={WifiOff}
        title={t.failed}
        description={timeline.error}
        actionLabel={c.retry}
        onAction={() => void run({query, mediaType, favorite, from, to})}
    />
{:else if timeline.loading && !searched}
    <div class="flex justify-center py-20">
        <div class="size-7 animate-spin rounded-full border-2 border-line border-t-primary"></div>
    </div>
{:else if searched && timeline.items.length === 0}
    <EmptyState icon={Search} title={t.noMatch} actionLabel={t.clearFilters} onAction={clearFilters} />
{:else if !searched}
    <EmptyState icon={Search} title={t.idleTitle} />
    {#if recent.length > 0}
        <div class="mx-auto max-w-3xl px-4">
            <p class="mb-2 px-1 text-xs font-medium text-muted">{t.recent}</p>
            <div class="flex flex-wrap gap-2">
                {#each recent as term (term)}
                    <button
                        onclick={() => applyRecent(term)}
                        class="rounded-full bg-gray px-3 py-1.5 text-xs text-muted transition-colors hover:text-dark"
                    >
                        {term}
                    </button>
                {/each}
            </div>
        </div>
    {/if}
{:else}
    <div class="mt-3 flex items-center justify-between px-4 text-xs text-muted">
        <span>{t.results(timeline.items.length, Boolean(timeline.next))}</span>
        {#if timeline.loading}<span>{t.searching}</span>{/if}
    </div>
    <AssetGrid
        items={timeline.items}
        columns={app.columns}
        selectionMode={selection.active}
        selected={selection.ids}
        onOpen={open}
        onToggle={(asset) => selection.toggle(asset.id)}
        onColumns={(columns) => app.setColumns(columns)}
    />
    <LoadMoreSentinel hasMore={timeline.next !== null} onLoad={() => void timeline.loadMore()} />
{/if}

{#if selection.active}
    <SelectionBar
        count={selection.count}
        isBusy={moving}
        onMove={() => (pickerOpen = true)}
        onDownload={() => void downloadSelected()}
        onCancel={() => selection.exit()}
    />
{/if}

<AlbumPickerSheet bind:open={pickerOpen} onPick={(album) => void moveToAlbum(album)} />

{#if viewerOpen}
    <TimelineViewer {timeline} bind:index={viewerIndex} onClose={() => (viewerOpen = false)} />
{/if}
