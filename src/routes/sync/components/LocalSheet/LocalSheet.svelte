<script lang="ts">
    import {Button, CloseButton} from '@immich/ui';
    import {mdiArrowTopRight, mdiLinkOff} from '@mdi/js';

    import MediaMosaic from '#lib/components/assets/MediaMosaic/MediaMosaic.svelte';
    import {folderMosaic} from '#lib/components/assets/MediaMosaic/mosaic';
    import Sheet from '#lib/components/overlay/Sheet/Sheet.svelte';
    import {common} from '#lib/i18n/common.i18n';
    import {i18n} from '#lib/i18n/i18n.svelte';
    import type {Mapping} from '../../sync.svelte';
    import TaskList from '../TaskList/TaskList.svelte';
    import {localSheet} from './LocalSheet.i18n';

    /// The folder as a source: where it is, which album it feeds, what it has
    /// queued, and how to stop feeding that album without touching the files.
    ///
    /// The sheet only asks; the page confirms, because a dialog opened from
    /// inside a sheet is stacked under it.
    let {
        mapping,
        open = $bindable(false),
        onOpen,
        onUnbind,
    }: {
        mapping: Mapping | null;
        open?: boolean;
        onOpen: () => void;
        onUnbind: () => void;
    } = $props();

    const t = $derived(i18n.pick(localSheet));
    const c = $derived(i18n.pick(common));
</script>

<Sheet bind:open label={t.localDetail} panelClass="max-h-[85vh] max-w-lg">
    {#if mapping}
        {@const local = mapping.local}
        <header class="flex items-start gap-3 border-b border-line/60 px-4 py-3">
            <div class="relative size-11 shrink-0 overflow-hidden rounded-xl bg-gray">
                <MediaMosaic sources={folderMosaic(local.coverUris)} iconSize={16} />
            </div>
            <div class="min-w-0 flex-1">
                <p class="truncate text-sm font-semibold leading-tight">{local.name}</p>
                <p class="truncate text-[11px] leading-tight text-muted">{local.albumName}</p>
            </div>
            <CloseButton color="secondary" onclick={() => (open = false)} />
        </header>
        <div class="min-h-0 flex-1 overflow-y-auto px-4 py-3">
            <dl class="grid grid-cols-[auto_minmax(0,1fr)] gap-x-3 gap-y-1.5 text-[11px]">
                <dt class="text-muted">{t.source}</dt>
                <dd class="truncate">{local.uri}</dd>
                <dt class="text-muted">{t.target}</dt>
                <dd class="truncate">{local.albumName}</dd>
                <dt class="text-muted">{t.files}</dt>
                <dd class="truncate">{c.items(local.assetCount)}</dd>
            </dl>
            <TaskList tasks={mapping.tasks} />
        </div>
        <div class="flex flex-col gap-2 border-t border-line/60 p-3">
            <Button
                color="primary"
                fullWidth
                leadingIcon={mdiArrowTopRight}
                onclick={() => {
                    open = false;
                    onOpen();
                }}
            >
                {t.openLocal}
            </Button>
            <Button
                variant="outline"
                color="danger"
                fullWidth
                leadingIcon={mdiLinkOff}
                onclick={() => {
                    open = false;
                    onUnbind();
                }}
            >
                {t.unbind}
            </Button>
        </div>
    {/if}
</Sheet>
