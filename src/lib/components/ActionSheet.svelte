<script lang="ts">
    import type {Component} from 'svelte';

    type ActionSheetItem = {
        label: string;
        icon: Component<{size?: number}>;
        danger?: boolean;
        onSelect: () => void;
    };

    let {
        open = $bindable(false),
        closeLabel,
        actions,
    }: {
        open?: boolean;
        closeLabel: string;
        actions: ActionSheetItem[];
    } = $props();

    const itemClass = 'flex w-full items-center gap-3 rounded-xl px-3 py-3 text-left text-sm hover:bg-gray';
</script>

{#if open}
    <div class="fixed inset-0 z-40 bg-black/40" role="presentation">
        <button
            type="button"
            class="absolute inset-0 size-full cursor-default"
            aria-label={closeLabel}
            onclick={() => (open = false)}
        ></button>
        <div
            class="absolute inset-x-3 bottom-[calc(1rem+var(--safe-bottom))] rounded-2xl bg-light p-2 shadow-2xl"
            role="menu"
            tabindex="-1"
        >
            {#each actions as action (action.label)}
                {@const Icon = action.icon}
                <button
                    role="menuitem"
                    onclick={() => {
                        open = false;
                        action.onSelect();
                    }}
                    class={action.danger ? `${itemClass} text-red-500` : itemClass}
                >
                    <Icon size={20} />
                    {action.label}
                </button>
            {/each}
        </div>
    </div>
{/if}
