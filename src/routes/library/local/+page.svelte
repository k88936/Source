<script lang="ts">
    import {onMount} from 'svelte';
    import {api} from '#lib/api/client';
    import type {AlbumSummary} from '#lib/api/bindings/AlbumSummary';
    import type {LocalAlbumSummary} from '#lib/api/bindings/LocalAlbumSummary';
    import type {LocalEntry} from '#lib/api/bindings/LocalEntry';
    import AlbumPickerSheet from '#lib/components/AlbumPickerSheet.svelte';
    import EmptyState from '#lib/components/EmptyState.svelte';
    import {ChevronRight, Folder, Plus, Upload} from '@lucide/svelte';
    import LocalDirPicker from '#lib/components/LocalDirPicker.svelte';
    import TopBar from '#lib/components/TopBar.svelte';
    import {common} from '#lib/i18n/common.i18n';
    import {app} from '#lib/state/app.svelte';
    import {i18n} from '#lib/state/i18n.svelte';
    import {local as text} from './page.i18n';

    let albums = $state<LocalAlbumSummary[]>([]);
    let loading = $state(true);
    let pickerOpen = $state(false);
    let targetOpen = $state(false);
    // The directory chosen in step one; step two asks for the album it syncs to.
    let pending = $state<LocalEntry | null>(null);
    let saving = $state(false);

    const t = $derived(i18n.pick(text));
    const c = $derived(i18n.pick(common));

    onMount(() => {
        void load();
    });

    async function load() {
        try {
            albums = await api.localAlbumList();
        } catch (err) {
            app.showToast(app.describeError(err), 'error');
        } finally {
            loading = false;
        }
    }

    function chose(entry: LocalEntry) {
        pending = entry;
        targetOpen = true;
    }

    /// A local album only exists to sync somewhere, so registering one and
    /// pointing it at an album are a single step.
    async function register(album: AlbumSummary) {
        const entry = pending;
        if (!entry || saving) return;
        saving = true;
        try {
            const created = await api.addLocalAlbum({
                uri: entry.uri,
                name: entry.name,
                albumId: album.id,
            });
            albums = [...albums.filter((row) => row.id !== created.id), created];
            app.showToast(t.synced(created.name, created.albumName), 'success');
        } catch (err) {
            app.showToast(app.describeError(err), 'error');
        } finally {
            saving = false;
            pending = null;
        }
    }
</script>

<svelte:head><title>{t.title} - Source</title></svelte:head>

<TopBar title={t.title} back>
    {#snippet actions()}
        <button
            onclick={() => (pickerOpen = true)}
            class="grid size-9 place-items-center rounded-full text-dark transition-colors hover:bg-gray"
            aria-label={c.syncLocalAlbum}
        >
            <Plus size={20} />
        </button>
    {/snippet}
</TopBar>

{#if loading}
    <div class="space-y-2 px-4 py-3">
        {#each Array(3) as _}
            <div class="h-16 animate-pulse rounded-2xl bg-gray"></div>
        {/each}
    </div>
{:else if albums.length === 0}
    <EmptyState
        icon={Folder}
        title={t.empty}
        description={t.emptyHint}
        actionLabel={c.syncLocalAlbum}
        onAction={() => (pickerOpen = true)}
    />
{:else}
    <ul class="space-y-2 px-4 py-3">
        {#each albums as album (album.id)}
            <li>
                <a
                    href="/library/{encodeURIComponent(album.id)}"
                    class="flex items-center gap-3 rounded-2xl bg-gray p-3 transition-colors hover:bg-line/40"
                >
                    <div class="grid size-11 shrink-0 place-items-center rounded-xl bg-primary/10 text-primary">
                        <Folder size={22} />
                    </div>
                    <div class="min-w-0 flex-1">
                        <p class="truncate text-sm font-semibold">{album.name}</p>
                        <p class="truncate text-xs text-muted">
                            {t.items(album.assetCount, album.albumName)}
                        </p>
                    </div>
                    <ChevronRight size={18} class="text-muted" />
                </a>
            </li>
        {/each}
    </ul>
{/if}

<LocalDirPicker bind:open={pickerOpen} onPick={chose} />
<AlbumPickerSheet bind:open={targetOpen} title={t.syncTo} onPick={(album) => void register(album)} />
