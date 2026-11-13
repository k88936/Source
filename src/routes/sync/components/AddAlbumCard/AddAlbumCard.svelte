<script lang="ts">
    import {Plus} from '@lucide/svelte';

    /// The cloud column ends with one card that adds an album there, after every
    /// album the library already has, so the list never grows upward. It keeps
    /// an album card's footprint as a dashed outline.
    let {onOpen}: {onOpen: () => void} = $props();
</script>

<button type="button" onclick={onOpen} class="group relative block w-full text-left">
    <div
        class="overflow-hidden rounded-xl border border-dashed border-line transition-colors group-hover:border-primary/50 group-hover:bg-primary/5"
    >
        <div class="aspect-[4/3]"></div>
        <div class="min-h-16"></div>
    </div>
    <span class="absolute inset-0 grid place-items-center text-muted transition-colors group-hover:text-primary">
        <Plus size={18} />
    </span>
</button>
