<script lang="ts">
    import {onMount} from 'svelte';
    import {useNavigation} from 'svelte-navigation';
    import {api} from '#lib/api/client';
    import type {AlbumSummary} from '#lib/api/bindings/AlbumSummary';
    import type {FolderRef} from '#lib/api/bindings/FolderRef';
    import AlbumCard from '#lib/components/albums/AlbumCard/AlbumCard.svelte';
    import EmptyState from '#lib/components/assets/EmptyState/EmptyState.svelte';
    import {Images, Plus, RefreshCw, Square, WifiOff} from '@lucide/svelte';
    import RemoteFolderPicker from '#lib/components/albums/RemoteFolderPicker/RemoteFolderPicker.svelte';
    import {common} from '#lib/i18n/common.i18n';
    import {app} from '#lib/app/app.svelte';
    import {changes} from '#lib/api/changes';
    import {i18n} from '#lib/i18n/i18n.svelte';
    import type {AlbumsNavParamList} from './route';
    import ScreenOptions from '#lib/components/shell/ScreenOptions/ScreenOptions.svelte';
    import {albums as text} from './Albums.i18n';

    const navigation = useNavigation<AlbumsNavParamList>();

    let albums = $state<AlbumSummary[]>([]);
    let loading = $state(true);
    let error = $state('');
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

<ScreenOptions title={c.albums} subtitle={loading ? c.loading : t.count(albums.length)} {actions} />

{#if loading && albums.length === 0}
    <div class="grid grid-cols-2 gap-3 p-4">
        {#each Array(4) as _}
            <div class="aspect-square animate-pulse rounded-2xl bg-gray"></div>
        {/each}
    </div>
{:else if error}
    <EmptyState
        icon={WifiOff}
        title={t.failed}
        description={error}
        actionLabel={c.retry}
        onAction={() => void load()}
    />
{:else if albums.length === 0}
    <EmptyState icon={Images} title={t.empty} actionLabel={c.addAlbum} onAction={() => (pickerOpen = true)} />
{:else}
    <div class="px-4 pb-24 pt-2">
        <div class="grid grid-cols-2 gap-3">
            {#each albums as album (album.id)}
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
