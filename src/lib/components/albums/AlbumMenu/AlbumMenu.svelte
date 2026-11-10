<script lang="ts">
    import ActionSheet from '#lib/components/overlay/ActionSheet/ActionSheet.svelte';
    import {common} from '#lib/i18n/common.i18n';
    import {i18n} from '#lib/i18n/i18n.svelte';
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
    const c = $derived(i18n.pick(common));

    const actions = $derived([
        {label: t.cover, icon: Image, onSelect: onCover},
        {label: t.description, icon: Info, onSelect: onDescription},
        {label: c.removeAlbum, icon: Trash, danger: true, onSelect: onRemove},
    ]);
</script>

<ActionSheet bind:open closeLabel={c.closeMenu} {actions} />
