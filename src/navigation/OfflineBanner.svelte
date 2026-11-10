<script lang="ts">
    import {common} from '#lib/i18n/common.i18n';
    import {app} from '#lib/state/app.svelte';
    import {i18n} from '#lib/state/i18n.svelte';

    const c = $derived(i18n.pick(common));
</script>

<!-- Sits above every header while the server is unreachable, so no screen repeats it. -->
{#if app.setup === 'offline' && app.account}
    <p class="flex h-6 items-center justify-center bg-gray text-[11px] text-muted">{c.offline}</p>
{/if}
