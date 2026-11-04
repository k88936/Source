<script lang="ts">
    import {api} from '#lib/api/client';
    import type {UploadTask} from '#lib/api/bindings/UploadTask';
    import {app} from '#lib/state/app.svelte';

    const active = $derived(app.uploads.filter((task) => !['done', 'cancelled'].includes(task.state)));
    const failed = $derived(active.filter((task) => task.state === 'failed'));

    function percent(task: UploadTask): number {
        if (task.total <= 0) return 0;
        return Math.min(100, Math.round((task.uploaded / task.total) * 100));
    }

    function label(task: UploadTask): string {
        switch (task.state) {
            case 'queued':
                return task.attempts > 0 ? `等待第 ${task.attempts + 1} 次尝试` : '等待中';
            case 'creatingSession':
                return '创建会话';
            case 'completing':
                return '收尾中';
            case 'failed':
                return '失败';
            default:
                return `${percent(task)}%`;
        }
    }

    async function cancel(taskId: string) {
        try {
            await api.cancelUpload({taskId});
            await app.refreshUploads();
        } catch (err) {
            app.showToast(app.describeError(err), 'error');
        }
    }

    async function retryAll() {
        for (const task of failed) await app.retryUpload(task.id);
    }
</script>

{#if active.length > 0}
    <section class="mt-3 rounded-2xl bg-gray p-4">
        <div class="mb-2 flex items-center justify-between">
            <p class="text-sm font-semibold">上传队列</p>
            <div class="flex items-center gap-2">
                {#if failed.length > 1}
                    <button
                        onclick={() => void retryAll()}
                        class="rounded-lg px-2 py-1 text-[11px] font-medium text-primary hover:bg-primary/10"
                    >
                        重试全部
                    </button>
                {/if}
                <span class="text-[11px] text-muted">{active.length} 项</span>
            </div>
        </div>
        <div class="flex flex-col gap-2">
            {#each active as task (task.id)}
                <div class="flex items-center gap-2">
                    <div class="min-w-0 flex-1">
                        <div class="flex justify-between gap-2 text-[11px]">
                            <span class="truncate">{task.name}</span>
                            <span class="shrink-0 {task.state === 'failed' ? 'text-red-500' : 'text-muted'}">
                                {label(task)}
                            </span>
                        </div>
                        {#if task.state !== 'failed'}
                            <div class="mt-1 h-1 overflow-hidden rounded-full bg-muted/25">
                                <div class="h-full rounded-full bg-primary" style:width="{percent(task)}%"></div>
                            </div>
                        {:else if task.error}
                            <p class="mt-0.5 truncate text-[10px] text-red-500">
                                {task.error.message}
                            </p>
                        {/if}
                    </div>
                    {#if task.state === 'failed'}
                        <button
                            onclick={() => void app.retryUpload(task.id)}
                            class="shrink-0 rounded-lg bg-primary px-2.5 py-1 text-[11px] font-medium text-on-primary"
                        >
                            重试
                        </button>
                    {:else}
                        <button
                            onclick={() => void cancel(task.id)}
                            class="shrink-0 text-[11px] text-muted hover:text-dark"
                        >
                            取消
                        </button>
                    {/if}
                </div>
            {/each}
        </div>
    </section>
{/if}
