<script lang="ts">
    import type {Component} from 'svelte';
    import Overlay from './Overlay.svelte';

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

<Overlay bind:open label={closeLabel}>
    <div class="pointer-events-none flex h-full flex-col justify-end px-3 pb-[calc(1rem+var(--safe-bottom))]">
        <div class="pointer-events-auto rounded-2xl bg-light p-2 shadow-2xl" role="menu" tabindex="-1">
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
</Overlay>
