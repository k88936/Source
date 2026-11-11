<script lang="ts">
    import {useNavigation} from 'svelte-navigation';
    import Viewer from './Viewer.svelte';
    import {viewer} from './viewer.svelte';

    const navigation = useNavigation();
</script>

{#if viewer.source}
    <Viewer
        items={viewer.source.items()}
        total={viewer.source.total()}
        onLoadMore={viewer.source.loadMore}
        actions={viewer.source.actions}
        bind:index={viewer.index}
        onClose={() => navigation.goBack()}
    />
{/if}
