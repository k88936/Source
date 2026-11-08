<script lang="ts">
    import ActionSheet from '#lib/components/ActionSheet.svelte';
    import {i18n} from '#lib/state/i18n.svelte';
    import {Trash} from '@lucide/svelte';
    import {menu} from './LocalAlbumMenu.i18n';

    let {
        open = $bindable(false),
        onUnlink,
    }: {
        open?: boolean;
        onUnlink: () => void;
    } = $props();

    const t = $derived(i18n.pick(menu));

    const actions = $derived([{label: t.unlink, icon: Trash, danger: true, onSelect: onUnlink}]);
</script>

<ActionSheet bind:open closeLabel={t.close} {actions} />
