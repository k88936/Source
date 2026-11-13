<script lang="ts">
    import {useNavigation} from 'svelte-navigation';
    import {api} from '#lib/api/client';
    import type {FolderRef} from '#lib/api/bindings/FolderRef';
    import {albumsView} from '#lib/api/views';
    import AlbumCard from './components/AlbumCard/AlbumCard.svelte';
    import EmptyState from '#lib/components/assets/EmptyState/EmptyState.svelte';
    import {Images, Plus, RefreshCw, Square, WifiOff} from '@lucide/svelte';
    import RemoteFolderPicker from './components/RemoteFolderPicker/RemoteFolderPicker.svelte';
    import {common} from '#lib/i18n/common.i18n';
    import {app} from '#lib/app/app.svelte';
    import {i18n} from '#lib/i18n/i18n.svelte';
    import type {AlbumsNavParamList} from './route';
    import ScreenOptions from '#lib/components/shell/ScreenOptions/ScreenOptions.svelte';
    import {albums as text} from './Albums.i18n';

    const navigation = useNavigation<AlbumsNavParamList>();

    // Live: one snapshot when the page subscribes, then the rows each catalog
    // write changed, so a running index pass keeps the counts and mosaics moving
    // without reloading the grid.
    const view = albumsView.use();
    let pickerOpen = $state(false);
    let adding = $state(false);

    const t = $derived(i18n.pick(text));
    const c = $derived(i18n.pick(common));
    // One album is one job, so the button is busy while any album of this kind
    // is still queued, paused or listing - there is no pass to ask about.
    const indexing = $derived(
        app.jobs.some(
            (job) =>
                job.kind === 'index' && (job.state === 'queued' || job.state === 'running' || job.state === 'paused'),
        ),
    );

    async function add(folder: FolderRef) {
        adding = true;
        try {
            // The row arrives through the subscription once the folder row is
            // committed, so the grid is not patched here.
            const album = await api.addAlbum({uri: folder.uri});
            app.showToast(t.added(album.name), 'success');
        } catch (err) {
            app.showToast(app.describeError(err), 'error');
        } finally {
            adding = false;
        }
    }

    async function refreshAll() {
        try {
            await api.indexStart();
        } catch (err) {
            app.showToast(app.describeError(err), 'error');
        }
    }

    async function stopRefresh() {
        try {
            await api.indexStop();
        } catch (err) {
            app.showToast(app.describeError(err), 'error');
        }
    }
</script>

<svelte:head><title>{c.albums} - Source</title></svelte:head>

{#snippet actions()}
    <button
        onclick={() => void (indexing ? stopRefresh() : refreshAll())}
        class="grid size-9 place-items-center rounded-full text-dark transition-colors hover:bg-gray"
        aria-label={indexing ? t.stopRefresh : t.refreshAll}
    >
        {#if indexing}
            <Square size={18} />
        {:else}
            <RefreshCw size={20} />
        {/if}
    </button>
    <button
        onclick={() => (pickerOpen = true)}
        class="grid size-9 place-items-center rounded-full text-dark transition-colors hover:bg-gray"
        aria-label={c.addAlbum}
    >
        <Plus size={22} />
    </button>
{/snippet}

<ScreenOptions title={c.albums} subtitle={view.loading ? c.loading : t.count(view.rows.length)} {actions} />

{#if view.loading && view.rows.length === 0}
    <div class="grid grid-cols-2 gap-3 p-4">
        {#each Array(4) as _, i (i)}
            <div class="aspect-square animate-pulse rounded-2xl bg-gray"></div>
        {/each}
    </div>
{:else if view.error}
    <EmptyState icon={WifiOff} title={t.failed} description={view.error} actionLabel={c.retry} onAction={view.retry} />
{:else if view.rows.length === 0}
    <EmptyState icon={Images} title={t.empty} actionLabel={c.addAlbum} onAction={() => (pickerOpen = true)} />
{:else}
    <div class="px-4 pb-24 pt-2">
        <div class="grid grid-cols-2 gap-3">
            {#each view.rows as album (album.id)}
                <AlbumCard {album} onOpen={() => navigation.navigate('Album', {id: album.id})} />
            {/each}
        </div>
    </div>
{/if}

<RemoteFolderPicker bind:open={pickerOpen} onPick={(folder) => void add(folder)} />

{#if adding}
    <div class="fixed inset-x-0 top-14 z-30 flex justify-center">
        <div class="rounded-full bg-dark/85 px-4 py-2 text-xs text-light">{t.adding}</div>
    </div>
{/if}
