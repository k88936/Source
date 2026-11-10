<script lang="ts">
    import TopBar from '#lib/components/TopBar.svelte';
    import {nav} from '#lib/i18n/nav.i18n';
    import {i18n} from '#lib/state/i18n.svelte';

    const t = $derived(i18n.pick(nav));
</script>

<svelte:head><title>{t.sync} - Source</title></svelte:head>

<TopBar title={t.sync} />
