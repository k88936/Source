<script lang="ts">
    let {
        open = $bindable(false),
        draft = $bindable(''),
        saving = false,
        onsave,
    }: {
        open?: boolean;
        draft?: string;
        saving?: boolean;
        onsave: () => void;
    } = $props();
</script>

{#if open}
    <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4" role="dialog" aria-modal="true">
        <div class="w-full max-w-md rounded-3xl bg-light p-5 shadow-2xl">
            <h2 class="text-lg font-medium">相册描述</h2>
            <p class="mt-1 text-xs text-muted">保存在 Cloudreve 目录 metadata: customize:album_description</p>
            <textarea bind:value={draft} rows="4" placeholder="写点什么..." class="field mt-3 resize-none"></textarea>
            <div class="mt-4 flex justify-end gap-2">
                <button
                    onclick={() => (open = false)}
                    class="rounded-xl px-4 py-2 text-sm font-medium text-muted hover:bg-gray"
                >
                    取消
                </button>
                <button
                    onclick={onsave}
                    disabled={saving}
                    class="rounded-xl bg-primary px-4 py-2 text-sm font-medium text-on-primary transition-opacity hover:opacity-90 disabled:opacity-50"
                >
                    {saving ? '保存中...' : '保存'}
                </button>
            </div>
        </div>
    </div>
{/if}
