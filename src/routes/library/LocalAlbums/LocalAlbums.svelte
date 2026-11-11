<script lang="ts">
    import {onMount} from 'svelte';
    import {useNavigation} from 'svelte-navigation';
    import {api} from '#lib/api/client';
    import type {AlbumMetadata} from '#lib/api/bindings/AlbumMetadata';
    import type {LocalAlbumMetadata} from '#lib/api/bindings/LocalAlbumMetadata';
    import type {LocalEntry} from '#lib/api/bindings/LocalEntry';
    import AlbumPickerSheet from '#lib/components/albums/AlbumPickerSheet/AlbumPickerSheet.svelte';
    import EmptyState from '#lib/components/assets/EmptyState/EmptyState.svelte';
    import {ChevronRight, Folder, Plus} from '@lucide/svelte';
    import LocalDirPicker from '../components/LocalDirPicker/LocalDirPicker.svelte';
    import {common} from '#lib/i18n/common.i18n';
    import {app} from '#lib/app/app.svelte';
    import {i18n} from '#lib/i18n/i18n.svelte';
    import type {LibraryNavParamList} from '../route';
    import ScreenOptions from '#lib/components/shell/ScreenOptions/ScreenOptions.svelte';
    import {local as text} from './LocalAlbums.i18n';

    let albums = $state<LocalAlbumMetadata[]>([]);
    let loading = $state(true);
    let pickerOpen = $state(false);
    let targetOpen = $state(false);
    // The directory chosen in step one; step two asks for the album it syncs to.
    let pending = $state<LocalEntry | null>(null);
    let saving = $state(false);

    const navigation = useNavigation<LibraryNavParamList>();

    const t = $derived(i18n.pick(text));
    const c = $derived(i18n.pick(common));

    // Returning from an album must show what that album did to the list.
    $effect(() => {
        return navigation.addListener('focus', () => void load());
    });

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
    async function register(album: AlbumMetadata) {
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

<svelte:head><title>{c.localAlbums} - Source</title></svelte:head>

{#snippet actions()}
    <button
        onclick={() => (pickerOpen = true)}
        class="grid size-9 place-items-center rounded-full text-dark transition-colors hover:bg-gray"
        aria-label={c.syncLocalAlbum}
    >
        <Plus size={20} />
    </button>
{/snippet}

<ScreenOptions title={c.localAlbums} {actions} />

{#if loading}
    <div class="space-y-2 px-4 py-3">
        {#each Array(3) as _, i (i)}
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
                <button
                    type="button"
                    onclick={() => navigation.navigate('LocalAlbum', {id: album.id})}
                    class="flex w-full items-center gap-3 rounded-2xl bg-gray p-3 text-left transition-colors hover:bg-line/40"
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
                </button>
            </li>
        {/each}
    </ul>
{/if}

<LocalDirPicker bind:open={pickerOpen} onPick={chose} />
<AlbumPickerSheet bind:open={targetOpen} title={t.syncTo} onPick={(album) => void register(album)} />
