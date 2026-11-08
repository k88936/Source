<script lang="ts">
    import type {Job} from '#lib/api/bindings/Job';
    import type {JobKind} from '#lib/api/bindings/JobKind';
    import type {JobState} from '#lib/api/bindings/JobState';
    import {app} from '#lib/state/app.svelte';
    import {i18n} from '#lib/state/i18n.svelte';
    import {queue} from './JobQueueCard.i18n';

    const t = $derived(i18n.pick(queue));

    /** Rows one kind lists before it only reports how many are left. */
    const ROWS = 3;

    function isLive(state: JobState): boolean {
        return state === 'queued' || state === 'running' || state === 'paused';
    }

    function percent(job: Job<JobKind>): number | null {
        const total = job.progress.total;
        if (!total) return null;
        return Math.min(100, Math.round((job.progress.done / total) * 100));
    }

    const groups = $derived(
        (['index', 'sync', 'upload'] as const).map((kind) => {
            const rows = app.jobs.filter((job) => job.kind === kind);
            return {
                kind,
                live: rows.filter((job) => isLive(job.state)),
                done: rows.filter((job) => job.state === 'done').length,
                failed: rows.filter((job) => job.state === 'failed').length,
                cancelled: rows.filter((job) => job.state === 'cancelled').length,
            };
        }),
    );
</script>

<section class="mt-3 rounded-2xl bg-gray p-4">
    <div class="mb-2 flex items-center justify-between">
        <p class="text-sm font-semibold">{t.title}</p>
        <span class="text-[11px] text-muted">{t.items(app.jobs.length)}</span>
    </div>
    {#if app.jobs.length === 0}
        <p class="py-1 text-xs text-muted">{t.empty}</p>
    {:else}
        <div class="flex flex-col gap-3">
            {#each groups as group (group.kind)}
                {#if group.live.length > 0 || group.done > 0 || group.failed > 0 || group.cancelled > 0}
                    <div>
                        <div class="flex items-baseline justify-between gap-2">
                            <p class="text-xs font-medium">{t.kind[group.kind]}</p>
                            <span class="shrink-0 text-[11px] text-muted">
                                {t.group(group.live.length, group.done, group.failed, group.cancelled)}
                            </span>
                        </div>
                        {#each group.live.slice(0, ROWS) as job (job.id)}
                            <div class="mt-2">
                                <p class="text-[11px]">{t.state[job.state]}</p>
                                {#if percent(job) !== null}
                                    <div class="mt-1 h-1 overflow-hidden rounded-full bg-muted/25">
                                        <div class="h-full rounded-full bg-primary" style:width="{percent(job)}%"></div>
                                    </div>
                                {:else}
                                    <p class="mt-0.5 text-[10px] text-muted">{t.processed(job.progress.done)}</p>
                                {/if}
                            </div>
                        {/each}
                        {#if group.live.length > ROWS}
                            <p class="mt-1 text-[10px] text-muted">{t.more(group.live.length - ROWS)}</p>
                        {/if}
                    </div>
                {/if}
            {/each}
        </div>
    {/if}
</section>
