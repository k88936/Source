<script lang="ts">
    import {useBackHandler, useIsFocused, useNavigation, useRoute, useScrollContainer} from 'svelte-navigation';
    import {api} from '#lib/api/client';
    import type {AlbumMetadata} from '#lib/api/bindings/AlbumMetadata';
    import type {AssetSummary} from '#lib/api/bindings/AssetSummary';
    import AlbumMenu from '../components/AlbumMenu/AlbumMenu.svelte';
    import AssetList from '#lib/components/assets/AssetList/AssetList.svelte';
    import ConfirmDialog from '#lib/components/overlay/ConfirmDialog/ConfirmDialog.svelte';
    import DescriptionDialog from '../components/DescriptionDialog/DescriptionDialog.svelte';
    import EmptyState from '#lib/components/assets/EmptyState/EmptyState.svelte';
    import {Ellipsis, Image, RefreshCw, Search as SearchIcon, WifiOff, X} from '@lucide/svelte';
    import SearchInput from '#lib/components/search/SearchInput/SearchInput.svelte';
    import SearchMode from '#lib/components/search/SearchMode/SearchMode.svelte';
    import SelectAllButton from '#lib/components/selection/SelectAllButton/SelectAllButton.svelte';
    import SelectionOverlay from '#lib/components/selection/SelectionOverlay/SelectionOverlay.svelte';
    import TimelineActions from '#lib/components/viewer/TimelineActions/TimelineActions.svelte';
    import {AssetBrowser} from '#lib/components/assets/browser.svelte';
    import {common} from '#lib/i18n/common.i18n';
    import {app} from '#lib/app/app.svelte';
    import {changes} from '#lib/api/changes';
    import {i18n} from '#lib/i18n/i18n.svelte';
    import type {AlbumsNavParamList} from '../route';
    import ScreenOptions from '#lib/components/shell/ScreenOptions/ScreenOptions.svelte';
    import {album as text} from './Album.i18n';

    const navigation = useNavigation<AlbumsNavParamList>();
    const focused = $derived(useIsFocused());
    const scroll = useScrollContainer();
    const takeBack = useBackHandler();
    const albumId = $derived(useRoute<AlbumsNavParamList>().params?.id ?? '');

    /** Confined to this album, so a result can never come from another one. */
    const browser = new AssetBrowser((cursor, limit) => api.albumPage({albumId, cursor, limit}), {
        scroll,
        folder: () => albumId,
    });

    const t = $derived(i18n.pick(text));
    const c = $derived(i18n.pick(common));

    let album = $state<AlbumMetadata | null>(null);
    let metaError = $state('');
    let menuOpen = $state(false);
    let removeOpen = $state(false);
    let coverMode = $state(false);
    let descriptionOpen = $state(false);
    let descriptionDraft = $state('');
    let saving = $state(false);
    let refreshing = $state(false);

    const title = $derived.by(() => {
        if (coverMode) return t.coverSelect;
        if (browser.selection.active) return c.selected(browser.selection.count);
        return album?.name ?? t.fallback;
    });

    const subtitle = $derived.by(() => {
        if (coverMode) return t.coverHint;
        if (browser.selection.active) return '';
        if (!album) return '';
        return t.items(album.assetCount, album.path);
    });

    $effect(() => {
        const id = albumId;
        browser.selection.exit();
        void load(id);
    });

    // The back key closes a mode before it leaves the album.
    $effect(() => {
        if (!coverMode && !browser.search.open && !browser.selection.active) return;
        return takeBack(() => {
            if (coverMode) coverMode = false;
            else browser.back();
        });
    });

    $effect(() => {
        return changes.subscribe((change) => {
            if (change.kind === 'albums') {
                const updated = change.albums.find((entry) => entry.id === albumId);
                if (updated) album = updated;
                if (browser.search.open) void browser.search.run();
                return;
            }
            for (const asset of change.assets) {
                if (asset.folderId === albumId) browser.timeline.upsert(asset);
            }
        });
    });

    $effect(() => () => browser.dispose());

    async function load(id: string, keepItems = false) {
        if (!keepItems) browser.timeline.reset();
        metaError = '';
        try {
            album = await api.albumGet({albumId: id});
            if (!album) {
                metaError = t.notFound;
                return;
            }
            descriptionDraft = album.description ?? '';
            await browser.timeline.reload();
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
            album = await api.refreshAlbum({albumId});
            await browser.timeline.reload();
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
        if (browser.selection.active) {
            browser.selection.toggle(asset.id);
            return;
        }
        browser.open(asset, navigation, viewerActions);
    }

    function onKeydown(event: KeyboardEvent) {
        // Escape leaves search, and only while this screen is the one on top.
        if (event.key !== 'Escape' || !focused || !browser.search.open || browser.selection.active) return;
        browser.search.exit();
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
</script>

<svelte:head><title>{album ? `${album.name} - ${t.fallback}` : t.fallback} - Source</title></svelte:head>

<svelte:window onkeydown={onKeydown} />

{#snippet searchContent()}
    <SearchInput search={browser.search} scoped />
{/snippet}

<!-- The viewer shows what it is given: the album injects the timeline controls. -->
{#snippet viewerActions({index}: {index: number})}
    <TimelineActions timeline={browser.shown} {index} />
{/snippet}

{#snippet actions()}
    {#if coverMode}
        <button
            onclick={() => (coverMode = false)}
            class="rounded-lg px-3 py-2 text-xs font-medium text-muted hover:bg-gray">{c.cancel}</button
        >
    {:else if browser.selection.active}
        <SelectAllButton selection={browser.selection} ids={browser.shown.items.map((item) => item.id)} />
        <button
            onclick={() => browser.selection.exit()}
            class="grid size-9 place-items-center rounded-full text-dark hover:bg-gray"
            aria-label={c.exitSelection}
        >
            <X size={21} />
        </button>
    {:else if browser.search.open}
        <button
            onclick={() => browser.search.exit()}
            class="rounded-full px-3 py-2 text-xs font-medium text-muted"
        >
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
            onclick={() => browser.search.enter()}
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

<ScreenOptions
    {title}
    {subtitle}
    content={browser.search.open ? searchContent : undefined}
    contentCentered
    {actions}
/>

{#if album?.sync?.failed && !coverMode && !browser.search.open}
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

{#if album?.description && !coverMode && !browser.search.open}
    <p class="whitespace-pre-wrap px-4 pt-2 text-sm text-muted">
        {album.description}
    </p>
{/if}

{#if browser.search.open}
    <SearchMode search={browser.search} selection={browser.selection} scoped onOpen={open} />
{:else if browser.timeline.loading && browser.timeline.items.length === 0 && !browser.timeline.loaded}
    <div class="grid grid-cols-3 gap-0.5">
        {#each Array(12) as _, i (i)}
            <div class="aspect-square animate-pulse bg-gray"></div>
        {/each}
    </div>
{:else if metaError || browser.timeline.error}
    <EmptyState
        icon={WifiOff}
        title={t.failed}
        description={metaError || browser.timeline.error}
        actionLabel={c.retry}
        onAction={() => void load(albumId)}
    />
{:else if browser.timeline.items.length === 0}
    <EmptyState
        icon={Image}
        title={t.empty}
        description={t.emptyHint}
        actionLabel={t.uploadPhotos}
        onAction={() => navigation.navigate('Library')}
    />
{:else}
    <AssetList
        timeline={browser.timeline}
        selection={browser.selection}
        columns={app.columns}
        onOpen={open}
        onColumns={(columns) => app.setColumns(columns)}
    />
{/if}

<SelectionOverlay
    selection={browser.selection}
    excludeId={albumId}
    onMoved={(ids) => browser.shown.remove(ids)}
/>

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
