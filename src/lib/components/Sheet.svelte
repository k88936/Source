<script lang="ts">
    import type {Snippet} from 'svelte';

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

{#if open}
    <div
        class="fixed inset-0 z-50 flex items-end justify-center bg-black/45 sm:items-center sm:p-6"
        role="dialog"
        aria-modal="true"
        aria-label={label}
        tabindex="-1"
        onclick={(event) => {
            if (event.target === event.currentTarget) open = false;
        }}
        onkeydown={(event) => {
            if (event.key !== 'Escape') return;
            // Escape belongs to the topmost dialog: closing this one must not
            // also reach a shortcut on the page underneath.
            event.stopPropagation();
            open = false;
        }}
    >
        <div
            class="flex w-full flex-col rounded-t-3xl bg-light pb-[var(--safe-bottom)] shadow-2xl sm:rounded-3xl {panelClass}"
        >
            {@render children()}
        </div>
    </div>
{/if}
