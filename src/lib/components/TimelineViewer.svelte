<script lang="ts">
    import type {Timeline} from '#lib/state/timeline.svelte';
    import Viewer from '#lib/components/Viewer.svelte';

    let {
        timeline,
        index = $bindable(0),
        onClose,
    }: {
        timeline: Timeline;
        index?: number;
        onClose: () => void;
    } = $props();
</script>

<Viewer
    items={timeline.items}
    total={timeline.total}
    onLoadMore={() => void timeline.loadMore()}
    bind:index
    {onClose}
/>
