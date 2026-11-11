<script lang="ts">
    import {CloudUpload} from '@lucide/svelte';
    import {useNavigation} from 'svelte-navigation';

    import type {AlbumMetadata} from '#lib/api/bindings/AlbumMetadata';
    import type {LocalAlbumMetadata} from '#lib/api/bindings/LocalAlbumMetadata';
    import type {LocalEntry} from '#lib/api/bindings/LocalEntry';
    import {app} from '#lib/app/app.svelte';
    import EmptyState from '#lib/components/assets/EmptyState/EmptyState.svelte';
    import ConfirmDialog from '#lib/components/overlay/ConfirmDialog/ConfirmDialog.svelte';
    import ScreenOptions from '#lib/components/shell/ScreenOptions/ScreenOptions.svelte';
    import {common} from '#lib/i18n/common.i18n';
    import {i18n} from '#lib/i18n/i18n.svelte';
    import LocalDirPicker from '../library/components/LocalDirPicker/LocalDirPicker.svelte';
    import AlbumSheet from './components/AlbumSheet/AlbumSheet.svelte';
    import LocalSheet from './components/LocalSheet/LocalSheet.svelte';
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

    /// The add card belongs to one album, so the picker already knows where the
    /// folder it returns will land.
    function addMapping(album: AlbumMetadata) {
        pickerAlbum = album;
        pickerOpen = true;
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

<svelte:head><title>{c.sync} - Source</title></svelte:head>

<ScreenOptions title={c.sync} />

<div class="mx-auto w-full max-w-2xl px-4 pb-4 pt-3">
    {#if board.albums.length === 0}
        <EmptyState icon={CloudUpload} title={t.empty} />
    {:else}
        <SyncGraph
            groups={board.groups}
            orphans={board.orphans}
            onAlbum={openAlbum}
            onLocal={openLocal}
            onAdd={addMapping}
        />
    {/if}
</div>

<!-- The overview hovers at the bottom of the viewport while the rows scroll
     under it. It stays a sibling after the content instead of being fixed, so
     the room it takes in the flow is the placeholder that lets the last row
     scroll clear of it, and the page still ends where the card ends. -->
<div class="sticky bottom-0 z-20 mx-auto w-full max-w-2xl px-4 pb-3">
    <SyncOverview queue={board.queue} />
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
