<script lang="ts">
    import {cubicOut} from 'svelte/easing';
    import {Tween} from 'svelte/motion';

    import {formatBytes} from '#lib/format';
    import {common} from '#lib/i18n/common.i18n';
    import {i18n} from '#lib/i18n/i18n.svelte';
    import type {Queue} from '../../sync.svelte';
    import {overview} from './SyncOverview.i18n';

    let {queue}: {queue: Queue} = $props();

    const t = $derived(i18n.pick(overview));
    const c = $derived(i18n.pick(common));

    const circumference = 2 * Math.PI * 40;

    /// The ring counts bytes, so it keeps creeping between task boundaries
    /// instead of jumping once per file.
    const shown = new Tween(0, {duration: 800, easing: cubicOut});
    $effect(() => {
        shown.set(queue.percent);
    });

    /// Speed and estimate on one line, joined here because the template trims
    /// the whitespace around the separator.
    const pace = $derived(`${formatBytes(queue.speed)}/s${queue.eta !== null ? ` · ~${queue.eta}s` : ''}`);
</script>

{#snippet pill(wrap: string, dot: string, value: number, label: string)}
    <span class="inline-flex items-center gap-1.5 rounded-full border px-2.5 py-1 text-[10px] {wrap}">
        <span class="size-1.5 rounded-full {dot}"></span>
        <span class="tabular-nums">{value}</span>
        {label}
    </span>
{/snippet}

<!-- The totals of the whole queue, for every album on the page at once. -->
<section
    class="relative overflow-hidden rounded-3xl border border-line/60 bg-gradient-to-br from-gray via-light to-light px-4 py-4 shadow-xl shadow-black/10 dark:shadow-black/40"
>
    <div
        class="pointer-events-none absolute -right-12 -top-20 size-52 rounded-full bg-primary/20 blur-3xl {queue.speed >
        0
            ? 'animate-pulse'
            : ''}"
    ></div>
    <div class="relative flex items-center gap-4">
        <div class="relative shrink-0">
            <svg viewBox="0 0 96 96" class="size-[88px] -rotate-90">
                <circle
                    cx="48"
                    cy="48"
                    r="40"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="7"
                    class="text-line/70"
                />
                <circle
                    cx="48"
                    cy="48"
                    r="40"
                    fill="none"
                    stroke="url(#sync-ring)"
                    stroke-width="7"
                    stroke-linecap="round"
                    stroke-dasharray={circumference}
                    stroke-dashoffset={circumference * (1 - shown.current)}
                />
                <defs>
                    <linearGradient id="sync-ring" x1="0" y1="0" x2="1" y2="1">
                        <stop offset="0%" stop-color="var(--color-primary-300)" />
                        <stop offset="100%" stop-color="var(--color-primary)" />
                    </linearGradient>
                </defs>
            </svg>
            <div class="absolute inset-0 grid place-items-center">
                <p class="text-lg font-semibold tabular-nums leading-none">
                    {Math.round(shown.current * 100)}<span class="text-[10px] text-muted">%</span>
                </p>
            </div>
        </div>
        <div class="min-w-0 flex-1">
            <p class="text-[10px] uppercase tracking-[0.18em] text-muted">{t.queue}</p>
            {#if queue.active}
                <p class="mt-1 truncate text-xl font-semibold tabular-nums leading-tight">
                    {formatBytes(queue.remaining)}
                    <span class="ml-1 text-[11px] font-normal text-muted">{c.remaining}</span>
                </p>
                <p class="mt-1 truncate text-[11px] tabular-nums text-muted">{pace}</p>
            {:else}
                <p class="mt-1 truncate text-lg font-semibold leading-tight">{t.idle}</p>
                <p class="mt-1 truncate text-[11px] tabular-nums text-muted">
                    {formatBytes(queue.totalBytes)}
                </p>
            {/if}
        </div>
    </div>
    <div class="relative mt-3 flex flex-wrap items-center gap-1.5">
        {@render pill('border-line/60 bg-light/70', 'bg-primary', queue.uploading, c.uploading)}
        {@render pill('border-line/60 bg-light/70', 'bg-muted', queue.waiting, c.waiting)}
        {#if queue.failed > 0}
            {@render pill(
                'border-danger/40 bg-danger-100/50 text-danger-800 dark:text-danger-300',
                'bg-danger-500',
                queue.failed,
                c.failed,
            )}
        {/if}
    </div>
</section>
