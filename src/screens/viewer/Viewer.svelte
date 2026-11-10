<script lang="ts">
    import {useNavigation} from 'svelte-navigation';
    import LocalViewer from '#lib/components/LocalViewer.svelte';
    import TimelineViewer from '#lib/components/TimelineViewer.svelte';
    import {viewerSource} from '#lib/state/viewer.svelte';

    const navigation = useNavigation();
</script>

{#if viewerSource.source?.kind === 'timeline'}
    <TimelineViewer
        timeline={viewerSource.source.timeline}
        bind:index={viewerSource.index}
        onClose={() => navigation.goBack()}
    />
{:else if viewerSource.source?.kind === 'local'}
    <LocalViewer
        items={viewerSource.source.items()}
        total={viewerSource.source.total()}
        onLoadMore={viewerSource.source.loadMore}
        bind:index={viewerSource.index}
        onClose={() => navigation.goBack()}
    />
{/if}
