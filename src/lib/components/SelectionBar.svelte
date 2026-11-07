<script lang="ts">
    import {common} from '#lib/i18n/common.i18n';
    import {i18n} from '#lib/state/i18n.svelte';
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

    const c = $derived(i18n.pick(common));
</script>

<div
    class="fixed inset-x-0 bottom-[calc(3.6rem+var(--safe-bottom))] z-20 border-t border-line bg-light/95 px-3 py-2 backdrop-blur sidebar:bottom-0 sidebar:ps-60"
>
    <div class="mx-auto flex max-w-3xl items-center justify-between gap-2 min-h-8 max-[380px]:gap-1">
        {#if onCancel}
            <button
                onclick={onCancel}
                class="shrink-0 rounded-xl px-2.5 py-2 text-sm text-muted hover:bg-gray max-[380px]:px-2 max-[380px]:text-xs"
            >
                {c.cancel}
            </button>
        {/if}
        <div class="flex shrink-0 items-center gap-1.5 max-[380px]:gap-1">
            {#if onMove}
                <button
                    onclick={onMove}
                    disabled={isBusy || count === 0}
                    class="flex shrink-0 items-center gap-1.5 whitespace-nowrap rounded-xl border border-line px-3 py-2 text-sm font-medium text-dark transition-colors hover:bg-gray disabled:opacity-40 max-[380px]:px-2 max-[380px]:text-xs"
                >
                    <Images size={16} />
                    {c.moveToAlbum}
                </button>
            {/if}
            {#if onDownload}
                <button
                    onclick={onDownload}
                    disabled={isBusy || count === 0}
                    class="flex shrink-0 items-center gap-1.5 whitespace-nowrap rounded-xl bg-primary px-3 py-2 text-sm font-medium text-on-primary transition-opacity hover:opacity-90 disabled:opacity-40 max-[380px]:px-2 max-[380px]:text-xs"
                >
                    <Download size={17} />
                    {c.download}
                </button>
            {/if}
        </div>
    </div>
</div>
