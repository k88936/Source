<script lang="ts">
    import {api} from '#lib/api/client';
    import type {LocalBrowsePage} from '#lib/api/bindings/LocalBrowsePage';
    import type {LocalEntry} from '#lib/api/bindings/LocalEntry';
    import {common} from '#lib/i18n/common.i18n';
    import {app} from '#lib/state/app.svelte';
    import {i18n} from '#lib/state/i18n.svelte';
    import {ChevronLeft, ChevronRight, Folder, X} from '@lucide/svelte';
    import {dirPicker} from './LocalDirPicker.i18n';

    /// Walks the device directories one level at a time. Desktop starts at the
    /// home directory and can go back; a phone library only offers a flat list
    /// of buckets, so there the same view is a picker: no parent, no descent.
    let {open = $bindable(false), onPick}: {open?: boolean; onPick?: (entry: LocalEntry) => void} = $props();

    const t = $derived(i18n.pick(dirPicker));
    const c = $derived(i18n.pick(common));

    let page = $state<LocalBrowsePage | null>(null);
    /// The bucket chosen out of the flat library list; the platform has no
    /// current directory to report, so the picker remembers the choice.
    let bucket = $state<LocalEntry | null>(null);
    let loading = $state(false);
    let error = $state('');

    /// No current directory and no parent means the platform handed over a
    /// list to pick from instead of a tree to walk.
    const flat = $derived(page !== null && page.current === null);

    $effect(() => {
        if (open) void browseTo(null);
    });

    async function browseTo(parent: string | null) {
        loading = true;
        error = '';
        try {
            page = await api.localBrowse({parent});
            bucket = page.current;
        } catch (err) {
            error = app.describeError(err);
        } finally {
            loading = false;
        }
    }

    function current(): LocalEntry | null {
        return page?.current ?? bucket;
    }

    function enter(entry: LocalEntry) {
        if (flat) {
            bucket = entry;
            return;
        }
        void browseTo(entry.uri);
    }

    function pick() {
        const entry = current();
        if (!entry) return;
        onPick?.(entry);
        open = false;
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
                    <h2 class="text-base font-semibold">{t.title}</h2>
                    <p class="truncate text-xs text-muted">{current()?.path ?? t.choose}</p>
                </div>
                <button
                    onclick={() => (open = false)}
                    class="grid size-9 place-items-center rounded-full bg-gray text-dark hover:bg-line/40"
                    aria-label={c.close}
                >
                    <X size={18} />
                </button>
            </div>

            <div class="flex-1 overflow-y-auto px-2">
                {#if page?.parent}
                    <button
                        onclick={() => void browseTo(page?.parent?.uri ?? null)}
                        class="mb-1 flex w-full items-center gap-3 rounded-xl px-3 py-2.5 text-left text-sm hover:bg-gray"
                    >
                        <ChevronLeft size={18} class="text-muted" />
                        {c.up}
                    </button>
                {/if}
                {#if loading}
                    <div class="flex justify-center py-12">
                        <div class="size-6 animate-spin rounded-full border-2 border-line border-t-primary"></div>
                    </div>
                {:else if error}
                    <p class="px-3 py-6 text-center text-xs text-red-500">{error}</p>
                {:else if page && page.entries.length === 0}
                    <p class="px-3 py-8 text-center text-xs text-muted">{flat ? t.emptyLibrary : t.empty}</p>
                {:else}
                    {#each page?.entries ?? [] as entry (entry.uri)}
                        <button
                            onclick={() => enter(entry)}
                            class="flex w-full items-center gap-3 rounded-xl px-3 py-2.5 text-left text-sm hover:bg-gray"
                            class:bg-gray={flat && current()?.uri === entry.uri}
                        >
                            <Folder size={20} class="text-amber-500" />
                            <span class="min-w-0 flex-1 truncate">{entry.name}</span>
                            <span class="max-w-[45%] truncate text-[11px] text-muted">{entry.path ?? ''}</span>
                            {#if !flat}
                                <ChevronRight size={15} class="text-muted" />
                            {/if}
                        </button>
                    {/each}
                {/if}
            </div>

            <div class="border-t border-line p-4 pb-[calc(1rem+var(--safe-bottom))]">
                <button
                    onclick={pick}
                    disabled={!current()}
                    class="h-11 w-full rounded-xl bg-primary text-sm font-semibold text-on-primary transition-opacity hover:opacity-90 disabled:opacity-40"
                >
                    {current() ? t.sync(current()?.name ?? '') : flat ? t.syncPromptLibrary : t.syncPrompt}
                </button>
            </div>
        </div>
    </div>
{/if}
