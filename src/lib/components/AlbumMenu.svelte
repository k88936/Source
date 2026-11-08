<script lang="ts">
    import ActionSheet from '#lib/components/ActionSheet.svelte';
    import {i18n} from '#lib/state/i18n.svelte';
    import {Image, Info, Trash} from '@lucide/svelte';
    import {menu} from './AlbumMenu.i18n';

    let {
        open = $bindable(false),
        onCover,
        onDescription,
        onRemove,
    }: {
        open?: boolean;
        onCover: () => void;
        onDescription: () => void;
        onRemove: () => void;
    } = $props();

    const t = $derived(i18n.pick(menu));

    const actions = $derived([
        {label: t.cover, icon: Image, onSelect: onCover},
        {label: t.description, icon: Info, onSelect: onDescription},
        {label: t.remove, icon: Trash, danger: true, onSelect: onRemove},
    ]);
</script>

<ActionSheet bind:open closeLabel={t.close} {actions} />
