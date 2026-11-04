<script lang="ts">
    import {api} from '#lib/api/client';
    import type {FolderRef} from '#lib/api/bindings/FolderRef';
    import type {RemoteFolderPage} from '#lib/api/bindings/RemoteFolderPage';
    import {app} from '#lib/state/app.svelte';
    import Icon from './Icon.svelte';

    /// Browses the remote folder tree and hands back the folder the user settled
    /// on. A folder is an album, so the button reads "add this folder".
    let {open = $bindable(false), onPick}: {open?: boolean; onPick?: (folder: FolderRef) => void} = $props();

    let browse = $state<RemoteFolderPage | null>(null);
    let loading = $state(false);
    let error = $state('');
    let creating = $state(false);
    let newName = $state('');
    let busy = $state(false);
    /// The drive root, where browsing starts.
    const ROOT_URI = 'cloudreve://my';
    /// The folders walked through to get here, so `上一级` climbs back out.
    let trail = $state<string[]>([]);

    $effect(() => {
        if (open) {
            trail = [];
            void browseTo(ROOT_URI);
        }
    });

    async function browseTo(uri: string) {
        loading = true;
        error = '';
        browse = null;
        try {
            browse = await api.listRemoteFolders({uri, pageToken: null});
        } catch (err) {
            error = app.describeError(err);
        } finally {
            loading = false;
        }
    }

    function enter(folder: FolderRef) {
        if (!browse) return;
        trail = [...trail, browse.current.uri];
        void browseTo(folder.uri);
    }

    function up() {
        const parent = trail.at(-1);
        if (parent === undefined) return;
        trail = trail.slice(0, -1);
        void browseTo(parent);
    }

    async function pick() {
        if (!browse || busy) return;
        busy = true;
        try {
            onPick?.(browse.current);
            open = false;
        } finally {
            busy = false;
        }
    }

    async function create() {
        const name = newName.trim();
        if (!name || !browse || busy) return;
        busy = true;
        error = '';
        try {
            const folder = await api.createFolder({
                parentUri: browse.current.uri,
                name,
            });
            newName = '';
            creating = false;
            onPick?.(folder);
            open = false;
        } catch (err) {
            error = app.describeError(err);
        } finally {
            busy = false;
        }
    }
</script>

{#if open}
    <div
        class="fixed inset-0 z-50 flex items-end justify-center bg-black/50 sm:items-center sm:p-4"
        role="dialog"
        aria-modal="true"
    >
        <div class="flex h-[80dvh] w-full max-w-lg flex-col rounded-t-3xl bg-light sm:h-[70dvh] sm:rounded-3xl">
            <div class="flex items-center justify-between px-4 pb-2 pt-3">
                <div class="min-w-0">
                    <h2 class="text-base font-semibold">添加相册</h2>
                    <p class="truncate text-xs text-muted">当前: {browse?.current.path ?? '/'}</p>
                </div>
                <div class="flex items-center gap-1">
                    <button
                        onclick={() => (creating = !creating)}
                        class="grid size-9 place-items-center rounded-full bg-gray text-dark hover:bg-line/40"
                        aria-label="新建文件夹"
                    >
                        <Icon name="folder" size={18} />
                    </button>
                    <button
                        onclick={() => (open = false)}
                        class="grid size-9 place-items-center rounded-full bg-gray text-dark hover:bg-line/40"
                        aria-label="关闭"
                    >
                        <Icon name="close" size={18} />
                    </button>
                </div>
            </div>

            {#if creating}
                <form
                    class="flex items-center gap-2 px-4 py-1"
                    onsubmit={(event) => {
                        event.preventDefault();
                        void create();
                    }}
                >
                    <input
                        bind:value={newName}
                        placeholder="新文件夹名"
                        class="min-w-0 flex-1 rounded-lg border border-line bg-transparent px-3 py-2 text-sm outline-none focus:ring-2 focus:ring-primary/60"
                    />
                    <button
                        type="submit"
                        disabled={!newName.trim() || busy}
                        class="rounded-lg bg-primary px-3 py-2 text-xs font-medium text-on-primary disabled:opacity-40"
                    >
                        创建
                    </button>
                </form>
            {/if}

            <div class="flex-1 overflow-y-auto px-2">
                {#if trail.length > 0}
                    <button
                        onclick={up}
                        class="mb-1 flex w-full items-center gap-3 rounded-xl px-3 py-2.5 text-left text-sm hover:bg-gray"
                    >
                        <Icon name="back" size={18} class="text-muted" /> 上一级
                    </button>
                {/if}
                {#if loading}
                    <div class="flex justify-center py-12">
                        <div class="size-6 animate-spin rounded-full border-2 border-line border-t-primary"></div>
                    </div>
                {:else if error}
                    <p class="px-3 py-6 text-center text-xs text-red-500">{error}</p>
                {:else if browse && browse.folders.length === 0}
                    <p class="px-3 py-8 text-center text-xs text-muted">这个目录下没有子文件夹</p>
                {:else}
                    {#each browse?.folders ?? [] as folder (folder.id)}
                        <button
                            onclick={() => enter(folder)}
                            class="flex w-full items-center gap-3 rounded-xl px-3 py-2.5 text-left text-sm hover:bg-gray"
                        >
                            <Icon name="folder" size={20} class="text-amber-500" />
                            <span class="min-w-0 flex-1 truncate">{folder.name}</span>
                            <Icon name="forward" size={15} class="text-muted" />
                        </button>
                    {/each}
                {/if}
            </div>

            <div class="border-t border-line p-4 pb-[calc(1rem+var(--safe-bottom))]">
                <button
                    onclick={() => void pick()}
                    disabled={!browse || busy}
                    class="h-11 w-full rounded-xl bg-primary text-sm font-semibold text-on-primary transition-opacity hover:opacity-90 disabled:opacity-40"
                >
                    {browse ? `添加 "${browse.current.name}" 为相册` : '加载中'}
                </button>
            </div>
        </div>
    </div>
{/if}
