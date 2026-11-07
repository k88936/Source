<script lang="ts">
    import {goto} from '$app/navigation';
    import {resolve} from '$app/paths';
    import {page} from '$app/state';
    import {api} from '#lib/api/client';
    import type {LocalAlbumSummary} from '#lib/api/bindings/LocalAlbumSummary';
    import type {LocalAsset} from '#lib/api/bindings/LocalAsset';
    import AssetThumb from '#lib/components/AssetThumb.svelte';
    import ConfirmDialog from '#lib/components/ConfirmDialog.svelte';
    import {Folder, Trash, Upload, X} from '@lucide/svelte';
    import TopBar from '#lib/components/TopBar.svelte';
    import UploadSheet from '#lib/components/UploadSheet.svelte';
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
    let sentinel: HTMLDivElement | undefined = $state();
    let selected = $state<string[]>([]);
    let uploadOpen = $state(false);
    let unlinkOpen = $state(false);
    // Uploads pick their target every time: the mapping records intent, it does
    // not drive the queue yet.
    let initialDest = $state('');
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

    $effect(() => {
        if (!sentinel || !next) return;
        const observer = new IntersectionObserver(
            (entries) => {
                if (entries[0]?.isIntersecting) void loadMore();
            },
            {rootMargin: '800px 0px'},
        );
        observer.observe(sentinel);
        return () => observer.disconnect();
    });

    async function load(id: string) {
        loading = true;
        next = null;
        try {
            const albums = await api.localAlbumList();
            album = albums.find((entry) => entry.id === id) ?? null;
            if (!album) return;
            const result = await api.localAssetPage({
                albumUri: album.uri,
                cursor: null,
                limit: PAGE,
            });
            items = result.items;
            next = result.next;
        } catch (err) {
            app.showToast(app.describeError(err), 'error');
        } finally {
            loading = false;
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

    function toggle(asset: LocalAsset) {
        selected = selected.includes(asset.id) ? selected.filter((id) => id !== asset.id) : [...selected, asset.id];
    }

    function openUpload() {
        initialDest = '';
        uploadOpen = true;
    }

    function uploaded() {
        selected = [];
        app.showToast(t.uploadCreated, 'success');
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
    title={selected.length ? t.selected(selected.length) : (album?.name ?? t.fallback)}
    subtitle={loading ? c.loading : t.subtitle(album?.assetCount ?? items.length, album?.albumName ?? '')}
    back={selected.length === 0}
>
    {#snippet actions()}
        {#if selected.length > 0}
            <button
                onclick={() => {
                    selected = [];
                }}
                class="grid size-9 place-items-center rounded-full text-dark hover:bg-gray"
                aria-label={t.cancelSelection}
            >
                <X size={21} />
            </button>
        {:else}
            <button
                onclick={() => (unlinkOpen = true)}
                class="grid size-9 place-items-center rounded-full text-dark hover:bg-gray"
                aria-label={t.unlink}
            >
                <Trash size={19} />
            </button>
            <button
                onclick={openUpload}
                class="grid size-9 place-items-center rounded-full text-dark hover:bg-gray"
                aria-label={c.upload}
            >
                <Upload size={21} />
            </button>
        {/if}
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
        <button onclick={openUpload} class="rounded-xl bg-primary px-4 py-2 text-xs font-medium text-on-primary"
            >{t.choosePhotos}</button
        >
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
                selectionMode
                selected={selected.includes(asset.id)}
                onToggle={() => toggle(asset)}
            />
        {/each}
    </div>

    {#if next}
        <div bind:this={sentinel} class="flex justify-center pb-24 pt-2">
            <div class="size-6 animate-spin rounded-full border-2 border-line border-t-primary"></div>
        </div>
    {/if}
{/if}

{#if selected.length > 0}
    <div
        class="fixed inset-x-0 bottom-[calc(3.6rem+var(--safe-bottom))] z-20 border-t border-line bg-light/95 px-4 py-2 backdrop-blur sidebar:bottom-0 sidebar:ps-60"
    >
        <div class="mx-auto flex max-w-3xl items-center justify-between">
            <span class="text-xs text-muted">{t.selectedCount(selected.length)}</span>
            <button
                onclick={openUpload}
                class="flex items-center gap-2 rounded-xl bg-primary px-4 py-2 text-sm font-medium text-on-primary"
            >
                <Upload size={17} />
                {t.uploadToAlbum}
            </button>
        </div>
    </div>
{/if}

<UploadSheet bind:open={uploadOpen} bind:initialDest initialSelection={selected} onUploaded={uploaded} />

<ConfirmDialog
    open={unlinkOpen}
    title={t.unlinkTitle(album?.name ?? '')}
    message={t.unlinkMessage}
    confirmLabel={t.unlink}
    danger
    onClose={() => (unlinkOpen = false)}
    onConfirm={() => void unlink()}
/>
