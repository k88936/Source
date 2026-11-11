<script lang="ts">
    import {Plus} from '@lucide/svelte';

    /// The add card belongs to one album, so the edge it draws goes to that one
    /// row and the picker it opens already knows where the folder will land.
    let {albumId, onOpen}: {albumId: string; onOpen: () => void} = $props();
</script>

<button
    type="button"
    data-edge={`add:${albumId}`}
    data-album={albumId}
    onclick={onOpen}
    class="group flex w-full items-center justify-center gap-2.5 rounded-xl border border-dashed border-line p-2 text-left text-muted transition-colors hover:border-primary/50 hover:bg-primary/5 hover:text-primary"
>
    <span class="grid size-9 shrink-0 place-items-center">
        <Plus size={16} />
    </span>
</button>
