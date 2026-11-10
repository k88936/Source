<script lang="ts">
    import type {AssetSummary} from '#lib/api/bindings/AssetSummary';
    import type {Search} from '../search.svelte';
    import type {Selection} from '../../selection/selection.svelte';
    import SearchFilters from '../SearchFilters/SearchFilters.svelte';
    import SearchResults from '../SearchResults/SearchResults.svelte';

    let {
        search,
        selection,
        scoped = false,
        onOpen,
    }: {
        search: Search;
        selection: Selection;
        /** Confined to one album, so the texts can say which library is searched. */
        scoped?: boolean;
        onOpen: (asset: AssetSummary) => void;
    } = $props();
</script>

{#if !selection.active}
    <SearchFilters {search} />
{/if}

<SearchResults {search} {selection} {scoped} {onOpen} />
