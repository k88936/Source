<script lang="ts">
    import type {Selection} from '#lib/components/selection/selection.svelte';
    import {i18n} from '#lib/i18n/i18n.svelte';
    import {selectAll} from './SelectAllButton.i18n';

    let {selection, ids}: {selection: Selection; ids: string[]} = $props();

    const t = $derived(i18n.pick(selectAll));
</script>

<button
    onclick={() => selection.toggleAll(ids)}
    disabled={ids.length === 0}
    class="rounded-lg px-3 py-2 text-xs font-medium text-primary disabled:opacity-40"
>
    {selection.allSelected(ids) ? t.none : t.all}
</button>
