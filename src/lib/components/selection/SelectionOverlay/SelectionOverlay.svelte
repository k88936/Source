<script lang="ts">
    import {api} from '#lib/api/client';
    import type {AlbumSummary} from '#lib/api/bindings/AlbumSummary';
    import {common} from '#lib/i18n/common.i18n';
    import {app} from '#lib/app/app.svelte';
    import {i18n} from '#lib/i18n/i18n.svelte';
    import type {Selection} from '../selection.svelte';
    import AlbumPickerSheet from '../../albums/AlbumPickerSheet/AlbumPickerSheet.svelte';
    import SelectionBar from '../SelectionBar/SelectionBar.svelte';

    let {
        selection,
        excludeId = '',
        onMoved,
    }: {
        selection: Selection;
        /** Album the assets already live in, hidden from the picker. */
        excludeId?: string;
        /** Lets the page patch its own list: photos keeps the assets, an album drops them. */
        onMoved?: (ids: string[]) => void;
    } = $props();

    const c = $derived(i18n.pick(common));

    let pickerOpen = $state(false);
    let moving = $state(false);

    async function download() {
        if (selection.count === 0) return;
        try {
            const result = await api.exportAssets({assetIds: selection.ids});
            app.showToast(c.exported(result.exported), 'success');
            selection.exit();
        } catch (err) {
            app.showToast(app.describeError(err), 'error');
        }
    }

    async function move(target: AlbumSummary) {
        const ids = [...selection.ids];
        moving = true;
        try {
            const moved = await api.moveAssetsToAlbum({assetIds: ids, albumId: target.id});
            onMoved?.(ids);
            app.showToast(c.moved(moved.length, target.name), 'success');
            selection.exit();
        } catch (err) {
            app.showToast(app.describeError(err), 'error');
        } finally {
            moving = false;
        }
    }
</script>

{#if selection.active}
    <SelectionBar
        count={selection.count}
        isBusy={moving}
        onMove={() => (pickerOpen = true)}
        onDownload={() => void download()}
        onCancel={() => selection.exit()}
    />
{/if}

<AlbumPickerSheet bind:open={pickerOpen} {excludeId} onPick={(album) => void move(album)} />
