<script lang="ts">
    import ActionSheet from '#lib/components/overlay/ActionSheet/ActionSheet.svelte';
    import {common} from '#lib/i18n/common.i18n';
    import {i18n} from '#lib/i18n/i18n.svelte';
    import {Trash} from '@lucide/svelte';

    let {
        open = $bindable(false),
        onUnlink,
    }: {
        open?: boolean;
        onUnlink: () => void;
    } = $props();

    const c = $derived(i18n.pick(common));

    const actions = $derived([{label: c.stopSync, icon: Trash, danger: true, onSelect: onUnlink}]);
</script>

<ActionSheet bind:open closeLabel={c.closeMenu} {actions} />
