<script lang="ts">
    import type {Selection} from '#lib/state/selection.svelte';

    let {selection, ids}: {selection: Selection; ids: string[]} = $props();
</script>

<button
    onclick={() => selection.toggleAll(ids)}
    disabled={ids.length === 0}
    class="rounded-lg px-3 py-2 text-xs font-medium text-primary disabled:opacity-40"
>
    {selection.allSelected(ids) ? '取消全选' : '全选'}
</button>
