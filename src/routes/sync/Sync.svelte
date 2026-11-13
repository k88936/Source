<script lang="ts">
    import {useNavigation} from 'svelte-navigation';

    import type {AlbumMetadata} from '#lib/api/bindings/AlbumMetadata';
    import type {FolderRef} from '#lib/api/bindings/FolderRef';
    import type {LocalAlbumMetadata} from '#lib/api/bindings/LocalAlbumMetadata';
    import type {LocalEntry} from '#lib/api/bindings/LocalEntry';
    import {api} from '#lib/api/client';
    import {app} from '#lib/app/app.svelte';
    import ConfirmDialog from '#lib/components/overlay/ConfirmDialog/ConfirmDialog.svelte';
    import ScreenOptions from '#lib/components/shell/ScreenOptions/ScreenOptions.svelte';
    import {common} from '#lib/i18n/common.i18n';
    import {i18n} from '#lib/i18n/i18n.svelte';
    import AlbumSheet from './components/AlbumSheet/AlbumSheet.svelte';
    import LocalDirPicker from './components/LocalDirPicker/LocalDirPicker.svelte';
    import LocalSheet from './components/LocalSheet/LocalSheet.svelte';
    import RemoteFolderPicker from './components/RemoteFolderPicker/RemoteFolderPicker.svelte';
    import SyncGraph from './components/SyncGraph/SyncGraph.svelte';
    import SyncOverview from './components/SyncOverview/SyncOverview.svelte';
    import type {SyncNavParamList} from './route';
    import {type Mapping, startSyncBoard, SyncBoard} from './sync.svelte';
    import {sync as text} from './Sync.i18n';

    const board = new SyncBoard();
    startSyncBoard(board);

    const t = $derived(i18n.pick(text));
    const c = $derived(i18n.pick(common));

    const navigation = useNavigation<SyncNavParamList>();

    /// Sheets hold ids, not the rows themselves: the rows are rebuilt on every
    /// tick, and a held object would freeze the modal while uploads move.
    let albumId = $state<string | null>(null);
    let albumOpen = $state(false);
    let localId = $state<string | null>(null);
    let localOpen = $state(false);

    let pickerOpen = $state(false);
    let pickerAlbum = $state<AlbumMetadata | null>(null);
    let albumPickerOpen = $state(false);
    let adding = $state(false);

    let unbindTarget = $state<LocalAlbumMetadata | null>(null);

    const selectedGroup = $derived(albumId ? (board.groups.find((group) => group.album.id === albumId) ?? null) : null);

    const selectedMapping: Mapping | null = $derived.by(() => {
        if (!localId) return null;
        for (const group of board.groups) {
            const found = group.mappings.find((mapping) => mapping.local.id === localId);
            if (found) return found;
        }
        return board.orphans.find((mapping) => mapping.local.id === localId) ?? null;
    });

    function openAlbum(album: AlbumMetadata) {
        albumId = album.id;
        albumOpen = true;
    }

    function openLocal(local: LocalAlbumMetadata) {
        localId = local.id;
        localOpen = true;
    }

    /// A mapping always belongs to one album, so the folder picker already knows
    /// where the folder it returns will land.
    function addMapping(album: AlbumMetadata) {
        pickerAlbum = album;
        pickerOpen = true;
    }

    /// The cloud column's card adds an album from a remote folder; the new row
    /// arrives through the catalog subscription.
    async function addAlbum(folder: FolderRef) {
        adding = true;
        try {
            const album = await api.addAlbum({uri: folder.uri});
            app.showToast(t.added(album.name), 'success');
        } catch (err) {
            app.showToast(app.describeError(err), 'error');
        } finally {
            adding = false;
        }
    }

    function openAlbumPage() {
        const album = selectedGroup?.album;
        albumOpen = false;
        if (album) navigation.navigate('Album', {id: album.id});
    }

    function openLocalPage() {
        const local = selectedMapping?.local;
        localOpen = false;
        if (local) navigation.navigate('LocalAlbum', {id: local.id});
    }

    async function attach(entry: LocalEntry) {
        const album = pickerAlbum;
        pickerAlbum = null;
        if (!album) return;
        try {
            await board.attach(entry.uri, entry.name, album.id);
            app.showToast(t.attached(entry.name, album.name), 'success');
        } catch (err) {
            app.showToast(app.describeError(err), 'error');
        }
    }

    async function unbind() {
        const local = unbindTarget;
        unbindTarget = null;
        if (!local) return;
        try {
            await board.unbind(local.id);
            app.showToast(t.unbound, 'success');
        } catch (err) {
            app.showToast(app.describeError(err), 'error');
        }
    }
</script>

<svelte:head><title>{c.albums} - Source</title></svelte:head>

<ScreenOptions title={c.albums} />

<div class="mx-auto w-full max-w-2xl px-4 pb-4">
    <!-- The overview is the page's dock: it opens at the top of the content and
         pins to the top of the viewport once the rows scroll under it, so it
         never leaves the screen and never eats a row. The top padding lives
         inside this dock, so the card keeps the same offset whether it is
         scrolling or pinned - padding on the page itself would be swallowed by
         the first scrolled pixel. -->
    <div class="sticky top-0 z-20 pb-3 pt-3">
        <SyncOverview queue={board.queue} />
    </div>

    <SyncGraph
        groups={board.groups}
        orphans={board.orphans}
        onAlbum={openAlbum}
        onLocal={openLocal}
        onAdd={addMapping}
        onAddAlbum={() => (albumPickerOpen = true)}
    />
</div>

<AlbumSheet bind:open={albumOpen} group={selectedGroup} onOpen={openAlbumPage} />

<LocalSheet
    bind:open={localOpen}
    mapping={selectedMapping}
    onOpen={openLocalPage}
    onUnbind={() => (unbindTarget = selectedMapping?.local ?? null)}
/>

<ConfirmDialog
    open={unbindTarget !== null}
    title={t.stopTitle(unbindTarget?.name ?? '')}
    message={t.stopMessage}
    confirmLabel={c.stopSync}
    danger
    onClose={() => (unbindTarget = null)}
    onConfirm={() => void unbind()}
/>

<LocalDirPicker bind:open={pickerOpen} destName={pickerAlbum?.name} onPick={(entry) => void attach(entry)} />

<RemoteFolderPicker bind:open={albumPickerOpen} onPick={(folder) => void addAlbum(folder)} />

{#if adding}
    <div class="fixed inset-x-0 top-14 z-30 flex justify-center">
        <div class="rounded-full bg-dark/85 px-4 py-2 text-xs text-light">{t.adding}</div>
    </div>
{/if}
