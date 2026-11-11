<script lang="ts">
    import {Badge, ProgressBar} from '@immich/ui';

    import type {QueuedUpload} from '#lib/api/bindings/QueuedUpload';
    import type {UploadState} from '#lib/api/bindings/UploadState';
    import {errorText} from '#lib/errors/i18n';
    import {formatBytes} from '#lib/format';
    import {i18n} from '#lib/i18n/i18n.svelte';
    import {taskList} from './TaskList.i18n';

    /// The files one folder has queued, newest state per row.
    let {tasks}: {tasks: QueuedUpload[]} = $props();

    const t = $derived(i18n.pick(taskList));

    function badgeColor(state: UploadState): 'primary' | 'success' | 'danger' | 'secondary' {
        switch (state) {
            case 'failed':
                return 'danger';
            case 'done':
                return 'success';
            case 'cancelled':
                return 'secondary';
            default:
                return 'primary';
        }
    }

    function progressColor(state: UploadState): 'primary' | 'success' | 'danger' {
        if (state === 'failed') return 'danger';
        if (state === 'done') return 'success';
        return 'primary';
    }
</script>

{#if tasks.length > 0}
    <div class="mt-2 flex flex-col gap-2">
        {#each tasks as task (task.id)}
            <div class="rounded-xl border border-line/60 bg-gray/30 p-3">
                <div class="flex items-center gap-2">
                    <p class="min-w-0 flex-1 truncate text-xs font-medium">{task.name}</p>
                    <Badge size="tiny" color={badgeColor(task.state)}>
                        {t.states[task.state]}
                    </Badge>
                </div>
                <p class="mt-1 truncate text-[10px] tabular-nums text-muted">
                    {formatBytes(task.uploaded)} / {formatBytes(task.total)}{#if task.speed > 0}
                        · {formatBytes(task.speed)}/s{/if}
                </p>
                <ProgressBar
                    class="mt-2"
                    value={task.uploaded}
                    max={Math.max(task.total, 1)}
                    size="tiny"
                    color={progressColor(task.state)}
                />
                {#if task.state === 'failed' && task.error}
                    <p class="mt-2 text-[10px] leading-snug text-danger-800 dark:text-danger-300">
                        {t.lastError}: {errorText(task.error)} · {t.attempt(task.attempts)}
                    </p>
                {/if}
            </div>
        {/each}
    </div>
{:else}
    <p class="mt-2 rounded-xl border border-line/60 bg-gray/30 px-3 py-6 text-center text-[11px] text-muted">
        {t.noTasks}
    </p>
{/if}
