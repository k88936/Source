<script lang="ts">
    import {onMount} from 'svelte';
    import {api} from '#lib/api/client';
    import type {AlbumSummary} from '#lib/api/bindings/AlbumSummary';
    import type {FolderRef} from '#lib/api/bindings/FolderRef';
    import AlbumCard from '#lib/components/AlbumCard.svelte';
    import EmptyState from '#lib/components/EmptyState.svelte';
    import {Images, Plus, RefreshCw, Square, WifiOff} from '@lucide/svelte';
    import RemoteFolderPicker from '#lib/components/RemoteFolderPicker.svelte';
    import TopBar from '#lib/components/TopBar.svelte';
    import {common} from '#lib/i18n/common.i18n';
    import {app} from '#lib/state/app.svelte';
    import {changes} from '#lib/state/changes';
    import {i18n} from '#lib/state/i18n.svelte';
    import {albums as text} from './page.i18n';

    let albums = $state<AlbumSummary[]>([]);
    let loading = $state(true);
    let error = $state('');
    let pickerOpen = $state(false);
    let adding = $state(false);
    let settled = $state('');
    // A pass restored from storage is already over, so it must not announce
    // itself; only a pass this page saw running can end while it is open.
    let observed = $state(false);

    const t = $derived(i18n.pick(text));
    const c = $derived(i18n.pick(common));
    const run = $derived(app.indexRun);
    const indexing = $derived(run?.state === 'running');

    $effect(() => {
        // Counts and mosaics come from assets too, so any catalog write refreshes
        // this grid; keyed each-blocks keep the scroll position intact.
        return changes.subscribe(() => void load());
    });

    $effect(() => {
        // A pass ends on its own, possibly long after the button was pressed.
        const current = run;
        if (!current) return;
        if (current.state === 'running') {
            observed = true;
            return;
        }
        if (!observed) return;
        const key = `${current.id}:${current.state}`;
        if (settled === key) return;
        settled = key;
        void load();
        if (current.state === 'cancelled') return;
        if (current.failed > 0) app.showToast(t.partialFailed(current.failed), 'error');
        else app.showToast(t.refreshed, 'success');
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
            // This page started the pass, so this page may announce its end; a
            // pass restored from storage stays silent.
            observed = true;
            app.indexRun = await api.indexStart();
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

<svelte:head><title>{t.title} - Source</title></svelte:head>

<TopBar title={t.title} subtitle={loading ? c.loading : t.count(albums.length)}>
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
            aria-label={t.addAlbum}
        >
            <Plus size={22} />
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
        icon={WifiOff}
        title={t.failed}
        description={error}
        actionLabel={c.retry}
        onAction={() => void load()}
    />
{:else if albums.length === 0}
    <EmptyState icon={Images} title={t.empty} actionLabel={t.addAlbum} onAction={() => (pickerOpen = true)} />
{:else}
    <div class="px-4 pb-24 pt-2">
        {#if indexing && run}
            <p class="mb-2 px-1 text-xs text-muted">
                {t.progress(run.done + run.failed, run.total)}
            </p>
        {/if}
        <div class="grid grid-cols-2 gap-3 sm:grid-cols-3 sidebar:grid-cols-4">
            {#each albums as album (album.id)}
                <AlbumCard {album} />
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
