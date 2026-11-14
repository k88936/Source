<script lang="ts">
    import {Button, CloseButton} from '@immich/ui';
    import {mdiArrowTopRight, mdiMagnifyScan, mdiStop} from '@mdi/js';

    import MediaMosaic from '#lib/components/assets/MediaMosaic/MediaMosaic.svelte';
    import {albumMosaic} from '#lib/components/assets/MediaMosaic/mosaic';
    import Sheet from '#lib/components/overlay/Sheet/Sheet.svelte';
    import {i18n} from '#lib/i18n/i18n.svelte';
    import {albums as text} from '../../Albums.i18n';
    import type {Group} from '../../albums.svelte.js';
    import {albumSheet} from './AlbumSheet.i18n';

    /// The album as a destination: what it is, whether it is being listed, and
    /// the two things that can be done to it from here - start or stop that
    /// listing, or leave the graph for the album page.
    ///
    /// `group` stays set while the sheet closes, so the content does not blink
    /// out from under the leaving panel.
    let {
        group,
        open = $bindable(false),
        onOpen,
        onStart,
        onCancel,
    }: {
        group: Group | null;
        open?: boolean;
        onOpen: () => void;
        onStart: () => void;
        onCancel: () => void;
    } = $props();

    const t = $derived(i18n.pick(albumSheet));
    const c = $derived(i18n.pick(text));

    /// The pass, while there is one: a row's state is the only progress an
    /// album has, because a listing knows its pages only as it walks them.
    const status = $derived.by(() => {
        const job = group?.job;
        if (!job) return null;
        if (job.state === 'queued') return c.queued;
        if (job.state === 'running') return c.indexing;
        return null;
    });

    /// The action is "stop" for exactly as long as the listing is still going.
    const listing = $derived(group?.job?.state === 'queued' || group?.job?.state === 'running');
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
                {#if status}
                    <p class="mt-0.5 truncate text-[11px] leading-tight text-primary">{status}</p>
                {/if}
            </div>
            <CloseButton color="secondary" onclick={() => (open = false)} />
        </header>
        <div class="flex flex-col gap-2 border-t border-line/60 p-3">
            {#if listing}
                <Button variant="outline" color="danger" fullWidth leadingIcon={mdiStop} onclick={onCancel}>
                    {c.cancelIndex}
                </Button>
            {:else}
                <Button variant="outline" color="primary" fullWidth leadingIcon={mdiMagnifyScan} onclick={onStart}>
                    {c.startIndex}
                </Button>
            {/if}
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
