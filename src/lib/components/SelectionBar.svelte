<script lang="ts">
    import {Download, Images} from '@lucide/svelte';

    let {
        count,
        isBusy = false,
        onMove,
        onDownload,
        onCancel,
    }: {
        count: number;
        isBusy?: boolean;
        onMove?: () => void;
        onDownload?: () => void;
        onCancel?: () => void;
    } = $props();
</script>

<div
    class="fixed inset-x-0 bottom-[calc(3.6rem+var(--safe-bottom))] z-20 border-t border-line bg-light/95 px-4 py-2 backdrop-blur sidebar:bottom-0 sidebar:ps-60"
>
    <div class="mx-auto flex max-w-3xl items-center justify-between gap-2 min-h-8">
        {#if onCancel}
            <button onclick={onCancel} class="rounded-xl px-3 py-2 text-sm text-muted hover:bg-gray"> 取消 </button>
        {/if}
        <div class="flex items-center gap-2">
            {#if onMove}
                <button
                    onclick={onMove}
                    disabled={isBusy || count === 0}
                    class="flex items-center gap-1.5 rounded-xl border border-line px-4 py-2 text-sm font-medium text-dark transition-colors hover:bg-gray disabled:opacity-40"
                >
                    <Images size={16} />
                    移动到相册
                </button>
            {/if}
            {#if onDownload}
                <button
                    onclick={onDownload}
                    disabled={isBusy || count === 0}
                    class="flex items-center gap-2 rounded-xl bg-primary px-4 py-2 text-sm font-medium text-on-primary transition-opacity hover:opacity-90 disabled:opacity-40"
                >
                    <Download size={17} />
                    下载
                </button>
            {/if}
        </div>
    </div>
</div>
