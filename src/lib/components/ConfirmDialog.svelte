<script lang="ts">
    import Icon from './Icon.svelte';

    let {
        open,
        title,
        message,
        confirmLabel = '确定',
        danger = false,
        onclose,
        onconfirm,
    }: {
        open: boolean;
        title: string;
        message: string;
        confirmLabel?: string;
        /** Destructive actions tint the icon and confirm button with the danger token. */
        danger?: boolean;
        onclose: () => void;
        onconfirm: () => void;
    } = $props();
</script>

<svelte:window
    onkeydown={(event) => {
        if (open && event.key === 'Escape') onclose();
    }}
/>

<!-- WebKitGTK's own `confirm()` is unthemeable and blocks the webview, so actions
     route through this dialog instead. -->
{#if open}
    <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4" role="dialog" aria-modal="true">
        <button type="button" class="absolute inset-0 cursor-default" aria-label="关闭" onclick={onclose}></button>
        <div class="relative w-full max-w-sm rounded-3xl bg-gray p-5 shadow-2xl ring-1 ring-line">
            <span
                class="grid size-10 place-items-center rounded-full {danger
                    ? 'bg-danger/10 text-danger'
                    : 'bg-primary/10 text-primary'}"
            >
                <Icon name={danger ? 'alert' : 'info'} size={20} />
            </span>
            <h2 class="mt-3 text-base font-semibold">{title}</h2>
            <p class="mt-1 whitespace-pre-line text-sm leading-relaxed text-muted">{message}</p>
            <div class="mt-5 flex justify-end gap-2">
                <button
                    onclick={onclose}
                    class="rounded-xl px-4 py-2 text-sm font-medium text-muted transition-colors hover:bg-line/40"
                >
                    取消
                </button>
                <button
                    onclick={onconfirm}
                    class="rounded-xl px-4 py-2 text-sm font-medium transition-colors {danger
                        ? 'bg-danger/10 text-danger hover:bg-danger/20'
                        : 'bg-primary text-on-primary hover:opacity-90'}"
                >
                    {confirmLabel}
                </button>
            </div>
        </div>
    </div>
{/if}
