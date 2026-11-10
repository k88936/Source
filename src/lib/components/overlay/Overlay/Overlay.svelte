<script lang="ts">
    import type {Snippet} from 'svelte';

    /// The only way into the browser's top layer. A native dialog outranks every
    /// z-index, so an overlay opened from a stack layer still covers the tab bar.
    /// The panel inside must be interactive; everything else dismisses on tap.
    let {
        open = $bindable(false),
        label,
        children,
    }: {
        open?: boolean;
        /** Names the dialog for assistive tech. */
        label: string;
        children: Snippet;
    } = $props();

    let dialog = $state<HTMLDialogElement>();

    $effect(() => {
        if (!open || !dialog || dialog.open) return;
        dialog.showModal();
        // Without focus inside, Escape never reaches the dialog's cancel event.
        dialog.focus();
    });
</script>

{#if open}
    <dialog
        bind:this={dialog}
        class="fixed inset-0 m-0 h-full max-h-none w-full max-w-none border-0 bg-transparent p-0 backdrop:bg-black/45"
        tabindex="-1"
        aria-label={label}
        oncancel={(event) => {
            // Escape belongs to the dialog: closing it here keeps `open` and the
            // top layer in step instead of leaving a closed dialog behind.
            event.preventDefault();
            open = false;
        }}
        onclick={(event) => {
            if (event.target === event.currentTarget) open = false;
        }}
    >
        {@render children()}
    </dialog>
{/if}
