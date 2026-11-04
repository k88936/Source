<script lang="ts">
    import {Image, Info, Trash} from '@lucide/svelte';
    import {i18n} from '#lib/state/i18n.svelte';
    import {menu} from './AlbumMenu.i18n';

    let {
        open = $bindable(false),
        onCover,
        onDescription,
        onRemove,
    }: {
        open?: boolean;
        onCover: () => void;
        onDescription: () => void;
        onRemove: () => void;
    } = $props();

    const t = $derived(i18n.pick(menu));

    const itemClass = 'flex w-full items-center gap-3 rounded-xl px-3 py-3 text-left text-sm hover:bg-gray';
</script>

{#if open}
    <div class="fixed inset-0 z-40 bg-black/40" role="presentation">
        <button
            type="button"
            class="absolute inset-0 size-full cursor-default"
            aria-label={t.close}
            onclick={() => (open = false)}
        ></button>
        <div
            class="absolute inset-x-3 bottom-[calc(1rem+var(--safe-bottom))] rounded-2xl bg-light p-2 shadow-2xl"
            role="menu"
            tabindex="-1"
        >
            <button
                onclick={() => {
                    open = false;
                    onCover();
                }}
                class={itemClass}
            >
                <Image size={20} />
                {t.cover}
            </button>
            <button
                onclick={() => {
                    open = false;
                    onDescription();
                }}
                class={itemClass}
            >
                <Info size={20} />
                {t.description}
            </button>
            <button
                onclick={() => {
                    open = false;
                    onRemove();
                }}
                class="flex w-full items-center gap-3 rounded-xl px-3 py-3 text-left text-sm text-red-500 hover:bg-gray"
            >
                <Trash size={20} />
                {t.remove}
            </button>
        </div>
    </div>
{/if}
