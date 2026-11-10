<script lang="ts">
    import {useBackHandler, useIsFocused, useNavigation, useRoute, useScrollContainer} from 'svelte-navigation';
    import {api} from '#lib/api/client';
    import type {AlbumSummary} from '#lib/api/bindings/AlbumSummary';
    import type {AssetSummary} from '#lib/api/bindings/AssetSummary';
    import AlbumMenu from '#lib/components/AlbumMenu.svelte';
    import AlbumPickerSheet from '#lib/components/AlbumPickerSheet.svelte';
    import AssetGrid from '#lib/components/AssetGrid.svelte';
    import ConfirmDialog from '#lib/components/ConfirmDialog.svelte';
    import DescriptionDialog from '#lib/components/DescriptionDialog.svelte';
    import EmptyState from '#lib/components/EmptyState.svelte';
    import LoadMoreSentinel from '#lib/components/LoadMoreSentinel.svelte';
    import {Ellipsis, Image, RefreshCw, Search as SearchIcon, WifiOff, X} from '@lucide/svelte';
    import SearchFilters from '#lib/components/SearchFilters.svelte';
    import SearchInput from '#lib/components/SearchInput.svelte';
    import SearchResults from '#lib/components/SearchResults.svelte';
    import SelectAllButton from '#lib/components/SelectAllButton.svelte';
    import SelectionBar from '#lib/components/SelectionBar.svelte';
    import {common} from '#lib/i18n/common.i18n';
    import {app} from '#lib/state/app.svelte';
    import {changes} from '#lib/state/changes';
    import {i18n} from '#lib/state/i18n.svelte';
    import {Search} from '#lib/state/search.svelte';
    import {Selection} from '#lib/state/selection.svelte';
    import {Timeline} from '#lib/state/timeline.svelte';
    import {viewerSource} from '#lib/state/viewer.svelte';
    import type {AlbumsNavParamList} from '#navigation/params';
    import ScreenOptions from '#navigation/ScreenOptions.svelte';
    import {album as text} from './Album.i18n';

    const navigation = useNavigation<AlbumsNavParamList>();
    const focused = $derived(useIsFocused());
    const scroll = useScrollContainer();
    const takeBack = useBackHandler();
    const albumId = $derived(useRoute<AlbumsNavParamList>().params?.id ?? '');

    const timeline = new Timeline((cursor, limit) => api.albumPage({albumId: albumId, cursor, limit}));
    const selection = new Selection();
    /** Confined to this album, so a result can never come from another one. */
    const search = new Search(() => albumId, selection, scroll);

    const t = $derived(i18n.pick(text));
    const c = $derived(i18n.pick(common));
    /** The list on screen: the timeline, or the results while search is on. */
    const shown = $derived(search.open ? search.timeline : timeline);

    let album = $state<AlbumSummary | null>(null);
    let metaError = $state('');
    let menuOpen = $state(false);
    let removeOpen = $state(false);
    let coverMode = $state(false);
    let descriptionOpen = $state(false);
    let descriptionDraft = $state('');
    let saving = $state(false);
    let pickerOpen = $state(false);
    let moving = $state(false);
    let refreshing = $state(false);

    const title = $derived.by(() => {
        if (coverMode) return t.coverSelect;
        if (selection.active) return t.selected(selection.count);
        return album?.name ?? t.fallback;
    });

    const subtitle = $derived.by(() => {
        if (coverMode) return t.coverHint;
        if (selection.active) return '';
        if (!album) return '';
        return t.items(album.assetCount, album.path);
    });

    $effect(() => {
        const id = albumId;
        selection.exit();
        void load(id);
    });

    // The back key closes a mode before it leaves the album.
    $effect(() => {
        if (!coverMode && !search.open && !selection.active) return;
        return takeBack(() => {
            if (coverMode) coverMode = false;
            else if (search.open) search.exit();
            else selection.exit();
        });
    });

    $effect(() => {
        return changes.subscribe((change) => {
            if (change.kind === 'albums') {
                const updated = change.albums.find((entry) => entry.id === albumId);
                if (updated) album = updated;
                if (search.open) void search.run();
                return;
            }
            for (const asset of change.assets) {
                if (asset.folderId === albumId) timeline.upsert(asset);
            }
        });
    });

    $effect(() => () => search.dispose());

    async function load(id: string, keepItems = false) {
        if (!keepItems) timeline.reset();
        metaError = '';
        try {
            album = await api.albumGet({albumId: id});
            if (!album) {
                metaError = t.notFound;
                return;
            }
            descriptionDraft = album.description ?? '';
            await timeline.reload();
        } catch (err) {
            metaError = app.describeError(err);
        }
    }

    /// List this folder once. A failure means the folder is gone or renamed on
    /// the server: the album stays, its assets stay, and `sync.failed` shows the
    /// banner offering a retry.
    async function refresh() {
        if (refreshing) return;
        refreshing = true;
        try {
            album = await api.refreshAlbum({albumId: albumId});
            await timeline.reload();
            app.showToast(t.refreshed, 'success');
        } catch (err) {
            await load(albumId, true);
            app.showToast(app.describeError(err), 'error');
        } finally {
            refreshing = false;
        }
    }

    async function remove() {
        if (!album) return;
        removeOpen = false;
        try {
            await api.removeAlbum({albumId: album.id});
            app.showToast(t.removed, 'success');
            navigation.goBack();
        } catch (err) {
            app.showToast(app.describeError(err), 'error');
        }
    }

    function open(asset: AssetSummary) {
        if (coverMode) {
            void setCover(asset);
            return;
        }
        if (selection.active) {
            selection.toggle(asset.id);
            return;
        }
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

    async function setCover(asset: AssetSummary) {
        if (!album) return;
        try {
            album = await api.setAlbumCover({albumId: album.id, assetId: asset.id});
            coverMode = false;
            app.showToast(t.coverSaved, 'success');
        } catch (err) {
            app.showToast(app.describeError(err), 'error');
        }
    }

    async function saveDescription() {
        if (!album) return;
        saving = true;
        try {
            album = await api.setAlbumDescription({
                albumId: album.id,
                description: descriptionDraft.trim(),
            });
            descriptionOpen = false;
            app.showToast(t.descriptionSaved, 'success');
        } catch (err) {
            app.showToast(app.describeError(err), 'error');
        } finally {
            saving = false;
        }
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

    async function moveToAlbum(target: AlbumSummary) {
        moving = true;
        const ids = [...selection.ids];
        try {
            const moved = await api.moveAssetsToAlbum({
                assetIds: ids,
                albumId: target.id,
            });
            // The assets left this album's folder, so drop them from the grid.
            shown.remove(ids);
            app.showToast(t.moved(moved.length, target.name), 'success');
            selection.exit();
        } catch (err) {
            app.showToast(app.describeError(err), 'error');
        } finally {
            moving = false;
        }
    }
</script>

<svelte:head><title>{album ? `${album.name} - ${t.fallback}` : t.fallback} - Source</title></svelte:head>

<svelte:window onkeydown={onKeydown} />

{#snippet searchContent()}
    <SearchInput {search} scoped />
{/snippet}

{#snippet actions()}
        {#if coverMode}
            <button
                onclick={() => (coverMode = false)}
                class="rounded-lg px-3 py-2 text-xs font-medium text-muted hover:bg-gray">{c.cancel}</button
            >
        {:else if selection.active}
            <SelectAllButton {selection} ids={shown.items.map((item) => item.id)} />
            <button
                onclick={() => selection.exit()}
                class="grid size-9 place-items-center rounded-full text-dark hover:bg-gray"
                aria-label={c.cancel}
            >
                <X size={21} />
            </button>
        {:else if search.open}
            <button onclick={() => search.exit()} class="rounded-full px-3 py-2 text-xs font-medium text-muted">
                {c.cancel}
            </button>
        {:else}
            <button
                onclick={() => void refresh()}
                class="grid size-9 place-items-center rounded-full text-dark hover:bg-gray"
                aria-label={t.refresh}
            >
                {#if refreshing}
                    <div class="size-4 animate-spin rounded-full border-2 border-line border-t-primary"></div>
                {:else}
                    <RefreshCw size={20} />
                {/if}
            </button>
            <button
                onclick={() => search.enter()}
                class="grid size-9 place-items-center rounded-full text-dark hover:bg-gray"
                aria-label={c.search}
            >
                <SearchIcon size={20} />
            </button>
            <button
                onclick={() => (menuOpen = true)}
                class="grid size-9 place-items-center rounded-full text-dark hover:bg-gray"
                aria-label={t.actions}
            >
                <Ellipsis size={22} />
            </button>
        {/if}
    {/snippet}

<ScreenOptions {title} {subtitle} content={search.open ? searchContent : undefined} {actions} />

{#if album?.sync?.failed && !coverMode && !search.open}
    <div class="mx-4 mt-3 rounded-2xl border border-amber-500/40 bg-amber-500/10 px-3 py-2.5">
        <p class="text-xs font-medium text-amber-600">{t.syncFailed}</p>
        <div class="mt-2 flex gap-2">
            <button
                onclick={() => void refresh()}
                class="rounded-lg bg-primary px-3 py-1.5 text-xs font-medium text-on-primary disabled:opacity-40"
                disabled={refreshing}>{c.retry}</button
            >
            <button
                onclick={() => (removeOpen = true)}
                class="rounded-lg border border-line px-3 py-1.5 text-xs text-muted hover:bg-gray"
                >{c.removeAlbum}</button
            >
        </div>
    </div>
{/if}

{#if album?.description && !coverMode && !search.open}
    <p class="whitespace-pre-wrap px-4 pt-2 text-sm text-muted">
        {album.description}
    </p>
{/if}

{#if search.open}
    {#if !selection.active}<SearchFilters {search} />{/if}
    <SearchResults {search} {selection} scoped onOpen={open} />
{:else if timeline.loading && timeline.items.length === 0 && !timeline.loaded}
    <div class="grid grid-cols-3 gap-0.5">
        {#each Array(12) as _}
            <div class="aspect-square animate-pulse bg-gray"></div>
        {/each}
    </div>
{:else if metaError || timeline.error}
    <EmptyState
        icon={WifiOff}
        title={t.failed}
        description={metaError || timeline.error}
        actionLabel={c.retry}
        onAction={() => void load(albumId)}
    />
{:else if timeline.items.length === 0}
    <EmptyState
        icon={Image}
        title={t.empty}
        description={t.emptyHint}
        actionLabel={t.uploadPhotos}
        onAction={() => navigation.navigate('Library')}
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

<AlbumPickerSheet bind:open={pickerOpen} excludeId={albumId} onPick={(target) => void moveToAlbum(target)} />

<AlbumMenu
    bind:open={menuOpen}
    onCover={() => (coverMode = true)}
    onDescription={() => (descriptionOpen = true)}
    onRemove={() => (removeOpen = true)}
/>

<ConfirmDialog
    open={removeOpen}
    title={t.removeTitle(album?.name ?? '')}
    message={t.removeMessage}
    confirmLabel={t.remove}
    danger
    onClose={() => (removeOpen = false)}
    onConfirm={() => void remove()}
/>

<DescriptionDialog
    bind:open={descriptionOpen}
    bind:draft={descriptionDraft}
    {saving}
    onSave={() => void saveDescription()}
/>
