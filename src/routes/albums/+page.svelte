<script lang="ts">
    import {onMount} from 'svelte';
    import {api} from '#lib/api/client';
    import type {AlbumSummary} from '#lib/api/bindings/AlbumSummary';
    import type {FolderRef} from '#lib/api/bindings/FolderRef';
    import AlbumCard from '#lib/components/AlbumCard.svelte';
    import EmptyState from '#lib/components/EmptyState.svelte';
    import Icon from '#lib/components/Icon.svelte';
    import RemoteFolderPicker from '#lib/components/RemoteFolderPicker.svelte';
    import TopBar from '#lib/components/TopBar.svelte';
    import {app} from '#lib/state/app.svelte';
    import {changes} from '#lib/state/changes';

    let albums = $state<AlbumSummary[]>([]);
    let loading = $state(true);
    let error = $state('');
    let pickerOpen = $state(false);
    let adding = $state(false);
    let refreshing = $state(false);

    $effect(() => {
        // Counts and mosaics come from assets too, so any catalog write refreshes
        // this grid; keyed each-blocks keep the scroll position intact.
        return changes.subscribe(() => void load());
    });

    onMount(() => {
        void load();
    });

    async function load() {
        error = '';
        try {
            albums = await api.albumList();
        } catch (err) {
            error = app.describeError(err);
        } finally {
            loading = false;
        }
    }

    async function add(folder: FolderRef) {
        adding = true;
        try {
            const album = await api.addAlbum({uri: folder.uri});
            albums = [album, ...albums.filter((entry) => entry.id !== album.id)];
            app.showToast(`已添加相册 "${album.name}"`, 'success');
        } catch (err) {
            app.showToast(app.describeError(err), 'error');
        } finally {
            adding = false;
        }
    }

    async function refreshAll() {
        if (refreshing) return;
        refreshing = true;
        try {
            albums = await api.refreshAllAlbums();
            app.showToast('相册已刷新', 'success');
        } catch (err) {
            app.showToast(app.describeError(err), 'error');
        } finally {
            refreshing = false;
        }
    }
</script>

<svelte:head><title>相册 - Source</title></svelte:head>

<TopBar title="相册" subtitle={loading ? '正在加载...' : `${albums.length} 个相册`}>
    {#snippet actions()}
        <button
            onclick={() => void refreshAll()}
            class="grid size-9 place-items-center rounded-full text-dark transition-colors hover:bg-gray"
            aria-label="刷新全部相册"
        >
            {#if refreshing}
                <div class="size-4 animate-spin rounded-full border-2 border-line border-t-primary"></div>
            {:else}
                <Icon name="sync" size={20} />
            {/if}
        </button>
        <button
            onclick={() => (pickerOpen = true)}
            class="grid size-9 place-items-center rounded-full text-dark transition-colors hover:bg-gray"
            aria-label="添加相册"
        >
            <Icon name="albums" size={22} />
        </button>
    {/snippet}
</TopBar>

{#if loading && albums.length === 0}
    <div class="grid grid-cols-2 gap-3 p-4">
        {#each Array(4) as _}
            <div class="aspect-square animate-pulse rounded-2xl bg-gray"></div>
        {/each}
    </div>
{:else if error}
    <EmptyState
        icon="wifi-off"
        title="无法加载相册"
        description={error}
        actionLabel="重试"
        onaction={() => void load()}
    />
{:else if albums.length === 0}
    <EmptyState icon="albums" title="还没有相册" actionLabel="添加相册" onaction={() => (pickerOpen = true)} />
{:else}
    <div class="px-4 pb-24 pt-2">
        <p class="mb-2 px-1 text-xs text-muted">相册就是云端的一个文件夹 - 只收录它直接包含的照片与视频</p>
        <div class="grid grid-cols-2 gap-3 sm:grid-cols-3 sidebar:grid-cols-4">
            {#each albums as album (album.id)}
                <AlbumCard {album} />
            {/each}
        </div>
    </div>
{/if}

<RemoteFolderPicker bind:open={pickerOpen} onpick={(folder) => void add(folder)} />

{#if adding}
    <div class="fixed inset-x-0 top-14 z-30 flex justify-center">
        <div class="rounded-full bg-dark/85 px-4 py-2 text-xs text-light">正在添加相册...</div>
    </div>
{/if}
