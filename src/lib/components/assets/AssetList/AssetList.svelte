<script lang="ts">
    import type {AssetSummary} from '#lib/api/bindings/AssetSummary';
    import type {Timeline} from '../timeline.svelte';
    import type {Selection} from '../../selection/selection.svelte';
    import AssetGrid from '../AssetGrid/AssetGrid.svelte';
    import LoadMoreSentinel from '../LoadMoreSentinel/LoadMoreSentinel.svelte';

    let {
        timeline,
        selection,
        columns,
        onOpen,
        onColumns,
    }: {
        timeline: Timeline;
        selection: Selection;
        columns: number;
        onOpen: (asset: AssetSummary) => void;
        onColumns: (columns: number) => void;
    } = $props();
</script>

<AssetGrid
    items={timeline.items}
    {columns}
    selectionMode={selection.active}
    selected={selection.ids}
    {onOpen}
    onToggle={(asset) => selection.toggle(asset.id)}
    {onColumns}
/>

<LoadMoreSentinel hasMore={timeline.next !== null} onLoad={() => void timeline.loadMore()} />
