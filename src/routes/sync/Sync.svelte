<script lang="ts">
    import {common} from '#lib/i18n/common.i18n';
    import {i18n} from '#lib/i18n/i18n.svelte';
    import ScreenOptions from '#lib/components/shell/ScreenOptions/ScreenOptions.svelte';

    const c = $derived(i18n.pick(common));
</script>

<svelte:head><title>{c.sync} - Source</title></svelte:head>

<ScreenOptions title={c.sync} />
