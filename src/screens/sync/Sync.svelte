<script lang="ts">
    import {nav} from '#lib/i18n/nav.i18n';
    import {i18n} from '#lib/state/i18n.svelte';
    import ScreenOptions from '#navigation/ScreenOptions.svelte';

    const t = $derived(i18n.pick(nav));
</script>

<svelte:head><title>{t.sync} - Source</title></svelte:head>

<ScreenOptions title={t.sync} />
