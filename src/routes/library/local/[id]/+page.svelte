<script lang="ts">
    import {goto} from '$app/navigation';
    import {resolve} from '$app/paths';
    import {page} from '$app/state';
    import {api} from '#lib/api/client';
    import type {LocalAlbumSummary} from '#lib/api/bindings/LocalAlbumSummary';
    import type {LocalAsset} from '#lib/api/bindings/LocalAsset';
    import AssetThumb from '#lib/components/AssetThumb.svelte';
    import ConfirmDialog from '#lib/components/ConfirmDialog.svelte';
    import LoadMoreSentinel from '#lib/components/LoadMoreSentinel.svelte';
    import LocalAlbumMenu from '#lib/components/LocalAlbumMenu.svelte';
    import LocalViewer from '#lib/components/LocalViewer.svelte';
    import {Ellipsis, Folder, RefreshCw} from '@lucide/svelte';
    import TopBar from '#lib/components/TopBar.svelte';
    import {formatBytes} from '#lib/format';
    import {common} from '#lib/i18n/common.i18n';
    import {app} from '#lib/state/app.svelte';
    import {i18n} from '#lib/state/i18n.svelte';
    import {localAlbum as text} from './page.i18n';

    const PAGE = 200;

    let album = $state<LocalAlbumSummary | null>(null);
    let items = $state<LocalAsset[]>([]);
    let next = $state<string | null>(null);
    let loading = $state(true);
    let loadingMore = $state(false);
    let refreshing = $state(false);
    let menuOpen = $state(false);
    let unlinkOpen = $state(false);
    let viewerOpen = $state(false);
    let viewerIndex = $state(0);
    let currentId = $state('');

    const t = $derived(i18n.pick(text));
    const c = $derived(i18n.pick(common));

    $effect(() => {
        const id = page.params.id ?? '';
        if (id && id !== currentId) {
            currentId = id;
            void load(id);
        }
    });

    async function load(id: string): Promise<boolean> {
        loading = true;
        next = null;
        try {
            const albums = await api.localAlbumList();
            album = albums.find((entry) => entry.id === id) ?? null;
            if (!album) return false;
            const result = await api.localAssetPage({
                albumUri: album.uri,
                cursor: null,
                limit: PAGE,
            });
            items = result.items;
            next = result.next;
            return true;
        } catch (err) {
            app.showToast(app.describeError(err), 'error');
            return false;
        } finally {
            loading = false;
        }
    }

    /// Re-reads the folder from the device: the page never caches a listing.
    async function refresh() {
        if (refreshing) return;
        refreshing = true;
        try {
            if (await load(currentId)) app.showToast(t.refreshed, 'success');
        } finally {
            refreshing = false;
        }
    }

    async function loadMore() {
        if (!next || loadingMore || !album) return;
        loadingMore = true;
        const id = currentId;
        const cursor = next;
        const uri = album.uri;
        try {
            const result = await api.localAssetPage({albumUri: uri, cursor, limit: PAGE});
            if (id !== currentId) return;
            items = [...items, ...result.items];
            next = result.next;
        } catch (err) {
            if (id === currentId) app.showToast(app.describeError(err), 'error');
        } finally {
            loadingMore = false;
        }
    }

    function open(asset: LocalAsset) {
        viewerIndex = items.findIndex((entry) => entry.id === asset.id);
        viewerOpen = true;
    }

    /// Unlink only. Files already on the server stay, and the album they went to
    /// is untouched.
    async function unlink() {
        if (!album) return;
        unlinkOpen = false;
        try {
            await api.removeLocalAlbum({id: album.id});
            app.showToast(t.unlinked, 'success');
            await goto(resolve('/library/local'));
        } catch (err) {
            app.showToast(app.describeError(err), 'error');
        }
    }
</script>

<svelte:head><title>{album?.name ?? t.fallback} - Source</title></svelte:head>

<TopBar
    title={album?.name ?? t.fallback}
    subtitle={loading ? c.loading : t.subtitle(album?.assetCount ?? items.length, album?.albumName ?? '')}
    back
>
    {#snippet actions()}
        <button
            onclick={() => void refresh()}
            disabled={refreshing}
            class="grid size-9 place-items-center rounded-full text-dark hover:bg-gray disabled:opacity-50"
            aria-label={t.refresh}
        >
            <RefreshCw size={20} class={refreshing ? 'animate-spin' : ''} />
        </button>
        <button
            onclick={() => (menuOpen = true)}
            class="grid size-9 place-items-center rounded-full text-dark hover:bg-gray"
            aria-label={t.actions}
        >
            <Ellipsis size={20} />
        </button>
    {/snippet}
</TopBar>

{#if loading}
    <div class="grid grid-cols-3 gap-[2px]">
        {#each Array(12) as _}
            <div class="aspect-square animate-pulse bg-gray"></div>
        {/each}
    </div>
{:else if items.length === 0}
    <div class="flex flex-col items-center gap-3 px-8 py-20 text-center">
        <Folder size={34} class="text-muted" />
        <p class="text-sm font-medium">{t.noPhotos}</p>
    </div>
{:else}
    <div class="grid grid-cols-3 gap-[2px] sidebar:grid-cols-5">
        {#each items as asset (asset.id)}
            {@const video = asset.mime?.startsWith('video') ?? false}
            <AssetThumb
                source={{kind: 'localThumb', uri: asset.uri}}
                name={asset.name}
                {video}
                badge={formatBytes(asset.size)}
                onOpen={() => open(asset)}
            />
        {/each}
    </div>

    <LoadMoreSentinel hasMore={next !== null} onLoad={() => void loadMore()} />
{/if}

{#if viewerOpen}
    <LocalViewer
        {items}
        total={album?.assetCount ?? items.length}
        onLoadMore={() => void loadMore()}
        bind:index={viewerIndex}
        onClose={() => (viewerOpen = false)}
    />
{/if}

<LocalAlbumMenu bind:open={menuOpen} onUnlink={() => (unlinkOpen = true)} />

<ConfirmDialog
    open={unlinkOpen}
    title={t.unlinkTitle(album?.name ?? '')}
    message={t.unlinkMessage}
    confirmLabel={t.unlink}
    danger
    onClose={() => (unlinkOpen = false)}
    onConfirm={() => void unlink()}
/>
