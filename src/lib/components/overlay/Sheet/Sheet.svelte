<script lang="ts">
    import type {Snippet} from 'svelte';
    import Overlay from '../Overlay/Overlay.svelte';

    /// The common shell of every sheet. It owns the backdrop, the close wiring
    /// and the bottom safe-area inset, so a panel only describes its own size.
    let {
        open = $bindable(false),
        label,
        panelClass = '',
        children,
    }: {
        open?: boolean;
        /** Names the dialog for assistive tech. */
        label: string;
        /** Sizing such as `max-w-lg`, applied to the panel. */
        panelClass?: string;
        children: Snippet;
    } = $props();
</script>

<Overlay bind:open {label}>
    <div class="pointer-events-none flex h-full items-end justify-center sm:items-center sm:p-6">
        <div
            class="pointer-events-auto flex w-full flex-col rounded-t-3xl bg-light pb-[var(--safe-bottom)] shadow-2xl sm:rounded-3xl {panelClass}"
        >
            {@render children()}
        </div>
    </div>
</Overlay>
