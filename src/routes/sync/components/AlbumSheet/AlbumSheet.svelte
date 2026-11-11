<script lang="ts">
    import {Button, CloseButton} from '@immich/ui';
    import {mdiArrowTopRight} from '@mdi/js';

    import MediaMosaic from '#lib/components/assets/MediaMosaic/MediaMosaic.svelte';
    import {albumMosaic} from '#lib/components/assets/MediaMosaic/mosaic';
    import Sheet from '#lib/components/overlay/Sheet/Sheet.svelte';
    import {i18n} from '#lib/i18n/i18n.svelte';
    import type {Group} from '../../sync.svelte';
    import QueueSummary from '../QueueSummary/QueueSummary.svelte';
    import {albumSheet} from './AlbumSheet.i18n';

    /// The album as a destination: what it is, what its queue is doing, and the
    /// one action that leaves the graph for the album page.
    ///
    /// `group` stays set while the sheet closes, so the content does not blink
    /// out from under the leaving panel.
    let {group, open = $bindable(false), onOpen}: {group: Group | null; open?: boolean; onOpen: () => void} = $props();

    const t = $derived(i18n.pick(albumSheet));
</script>

<Sheet bind:open label={t.albumDetail} panelClass="max-h-[88vh] max-w-lg">
    {#if group}
        <header class="flex items-start gap-3 border-b border-line/60 px-4 py-3">
            <div class="relative size-11 shrink-0 overflow-hidden rounded-xl bg-gray">
                <MediaMosaic sources={albumMosaic(group.album)} iconSize={16} />
            </div>
            <div class="min-w-0 flex-1">
                <p class="truncate text-sm font-semibold leading-tight">{group.album.name}</p>
                <p class="truncate text-[11px] leading-tight text-muted">{group.album.path}</p>
            </div>
            <CloseButton color="secondary" onclick={() => (open = false)} />
        </header>
        <div class="min-h-0 flex-1 overflow-y-auto px-4 py-3">
            <QueueSummary tally={group.tally} eta={group.eta} label={t.albumQueue} />
        </div>
        <div class="border-t border-line/60 p-3">
            <Button
                color="primary"
                fullWidth
                leadingIcon={mdiArrowTopRight}
                onclick={() => {
                    open = false;
                    onOpen();
                }}
            >
                {t.openAlbum}
            </Button>
        </div>
    {/if}
</Sheet>
