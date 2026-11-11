<script lang="ts">
    import type {Snippet} from 'svelte';
    import {useNavigation} from 'svelte-navigation';

    /**
     * The header of the screen that renders this. `navigation.setOptions` is the
     * imperative way; a screen whose title arrives late reads better as this tag,
     * because every field is a normal reactive prop.
     */
    let {
        title,
        subtitle,
        content,
        contentCentered = false,
        actions,
    }: {
        title?: string;
        subtitle?: string;
        content?: Snippet;
        /** Centres `content` between equal-width sides. */
        contentCentered?: boolean;
        actions?: Snippet;
    } = $props();

    const navigation = useNavigation();

    $effect(() => {
        navigation.setOptions({
            title,
            subtitle,
            headerContent: content,
            headerContentCentered: contentCentered,
            headerActions: actions,
        });
    });
</script>
