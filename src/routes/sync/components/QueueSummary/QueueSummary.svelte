<script lang="ts">
    import {ProgressBar} from '@immich/ui';

    import {formatBytes} from '#lib/format';
    import {common} from '#lib/i18n/common.i18n';
    import {i18n} from '#lib/i18n/i18n.svelte';
    import type {Tally} from '../../sync.svelte';

    /// One album's bytes: the bar, what is left, and what is moving it.
    let {tally, eta, label}: {tally: Tally; eta: number | null; label: string} = $props();

    const c = $derived(i18n.pick(common));
</script>

<div class="rounded-xl border border-line/60 bg-gray/30 p-3">
    <div class="flex items-baseline justify-between gap-2">
        <p class="text-[11px] text-muted">{label}</p>
        <p class="text-[11px] tabular-nums text-muted">{Math.round(tally.percent * 100)}%</p>
    </div>
    <ProgressBar
        class="mt-2"
        value={tally.percent}
        max={1}
        size="tiny"
        color={tally.failed > 0 && tally.uploading === 0 ? 'danger' : 'primary'}
    />
    <div class="mt-2 flex flex-wrap items-center gap-x-3 gap-y-1 text-[10px] leading-tight">
        {#if tally.uploading > 0}
            <span class="text-primary">{tally.uploading} {c.uploading}</span>
        {/if}
        {#if tally.waiting > 0}
            <span class="text-muted">{tally.waiting} {c.waiting}</span>
        {/if}
        {#if tally.failed > 0}
            <span class="text-danger-500">{tally.failed} {c.failed}</span>
        {/if}
        <span class="ml-auto tabular-nums text-muted">
            {formatBytes(Math.max(0, tally.totalBytes - tally.doneBytes))}
            {c.remaining}
        </span>
    </div>
    {#if tally.speed > 0}
        <p class="mt-1 text-[10px] tabular-nums leading-tight text-primary">
            {formatBytes(tally.speed)}/s{#if eta !== null}
                · ~{eta}s{/if}
        </p>
    {/if}
</div>
