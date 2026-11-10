<script lang="ts">
    import {onMount} from 'svelte';
    import {useBackHandler, useIsFocused, useNavigation, useScrollContainer} from 'svelte-navigation';
    import {api} from '#lib/api/client';
    import type {AlbumSummary} from '#lib/api/bindings/AlbumSummary';
    import type {AssetSummary} from '#lib/api/bindings/AssetSummary';
    import AlbumPickerSheet from '#lib/components/AlbumPickerSheet.svelte';
    import AssetGrid from '#lib/components/AssetGrid.svelte';
    import EmptyState from '#lib/components/EmptyState.svelte';
    import LoadMoreSentinel from '#lib/components/LoadMoreSentinel.svelte';
    import {Image, Search as SearchIcon, X} from '@lucide/svelte';
    import SearchFilters from '#lib/components/SearchFilters.svelte';
    import SearchInput from '#lib/components/SearchInput.svelte';
    import SearchResults from '#lib/components/SearchResults.svelte';
    import SelectAllButton from '#lib/components/SelectAllButton.svelte';
    import SelectionBar from '#lib/components/SelectionBar.svelte';
    import {common} from '#lib/i18n/common.i18n';
    import {app} from '#lib/state/app.svelte';
    import {i18n} from '#lib/state/i18n.svelte';
    import {changes} from '#lib/state/changes';
    import {Search} from '#lib/state/search.svelte';
    import {Selection} from '#lib/state/selection.svelte';
    import {Timeline} from '#lib/state/timeline.svelte';
    import {viewerSource} from '#lib/state/viewer.svelte';
    import type {PhotosNavParamList} from '#navigation/params';
    import ScreenOptions from '#navigation/ScreenOptions.svelte';
    import {photos} from './Photos.i18n';

    const navigation = useNavigation<PhotosNavParamList>();
    const focused = $derived(useIsFocused());
    const scroll = useScrollContainer();
    const takeBack = useBackHandler();

    const timeline = new Timeline((cursor, limit) => api.timelinePage({cursor, limit}));
    const selection = new Selection();
    const search = new Search(() => null, selection, scroll);

    const t = $derived(i18n.pick(photos));
    const c = $derived(i18n.pick(common));
    /** The list on screen: the timeline, or the results while search is on. */
    const shown = $derived(search.open ? search.timeline : timeline);

    let pickerOpen = $state(false);
    let moving = $state(false);

    $effect(() => {
        // Patch on writes, refetch only when a listing pass moved the ground.
        return changes.subscribe((change) => {
            if (change.kind === 'albums') {
                void timeline.reload();
                if (search.open) void search.run();
                return;
            }
            for (const asset of change.assets) timeline.upsert(asset);
        });
    });

    $effect(() => () => search.dispose());

    onMount(() => {
        void timeline.reload();
    });

    // The back key closes a mode before it leaves the timeline.
    $effect(() => {
        if (!search.open && !selection.active) return;
        return takeBack(() => {
            if (search.open) search.exit();
            else selection.exit();
        });
    });

    function open(asset: AssetSummary) {
        const index = shown.items.findIndex((item) => item.id === asset.id);
        if (index < 0) return;
        viewerSource.showTimeline(shown, index);
        navigation.navigate('Viewer');
    }

    function onKeydown(event: KeyboardEvent) {
        // Escape leaves search, and only while this screen is the one on top.
        if (event.key !== 'Escape' || !focused || !search.open || selection.active) return;
        search.exit();
    }

    async function downloadSelected() {
        if (selection.count === 0) return;
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
            for (const asset of moved) shown.upsert(asset);
            app.showToast(t.moved(moved.length, album.name), 'success');
            selection.exit();
        } catch (err) {
            app.showToast(app.describeError(err), 'error');
        } finally {
            moving = false;
        }
    }
</script>

<svelte:head><title>{t.title} - Source</title></svelte:head>

<svelte:window onkeydown={onKeydown} />

{#snippet searchContent()}
    <SearchInput {search} />
{/snippet}

{#snippet actions()}
        {#if selection.active}
            <SelectAllButton {selection} ids={shown.items.map((item) => item.id)} />
            <button
                onclick={() => selection.exit()}
                class="grid size-10 place-items-center rounded-full text-dark"
                aria-label={t.exitSelection}
            >
                <X size={21} />
            </button>
        {:else if search.open}
            <button
                onclick={() => search.exit()}
                class="rounded-full px-3 py-2 text-xs font-medium text-muted transition-colors hover:bg-gray"
                aria-label={c.cancel}
            >
                {c.cancel}
            </button>
        {:else}
            <button
                onclick={() => search.enter()}
                class="grid size-9 place-items-center rounded-full text-dark transition-colors hover:bg-gray"
                aria-label={c.search}
            >
                <SearchIcon size={20} />
            </button>
        {/if}
    {/snippet}

<ScreenOptions
    title={selection.active ? t.selected(selection.count) : t.title}
    subtitle={selection.active ? '' : (app.account?.siteName ?? '')}
    content={search.open && !selection.active ? searchContent : undefined}
    {actions}
/>

{#if search.open}
    {#if !selection.active}<SearchFilters {search} />{/if}
    <SearchResults {search} {selection} onOpen={open} />
{:else if !timeline.loaded}
    <div class="grid gap-[2px]" style:grid-template-columns="repeat({app.columns}, minmax(0, 1fr))">
        {#each Array(app.columns * 6) as _}
            <div class="aspect-square animate-pulse bg-gray"></div>
        {/each}
    </div>
{:else if timeline.items.length === 0 && !timeline.loading}
    <EmptyState
        icon={Image}
        title={timeline.error ? t.failed : t.empty}
        description={timeline.error}
        actionLabel={timeline.error ? c.retry : t.addAlbum}
        onAction={() => (timeline.error ? void timeline.reload() : navigation.navigate('Albums'))}
    />
{:else}
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
