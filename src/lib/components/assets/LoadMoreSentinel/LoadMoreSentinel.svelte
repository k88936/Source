<script lang="ts">
    let {hasMore, onLoad}: {hasMore: boolean; onLoad: () => void} = $props();

    let sentinel = $state<HTMLDivElement>();

    $effect(() => {
        if (!sentinel || !hasMore) return;
        const observer = new IntersectionObserver(
            (entries) => {
                if (entries[0]?.isIntersecting) onLoad();
            },
            {rootMargin: '800px 0px'},
        );
        observer.observe(sentinel);
        return () => observer.disconnect();
    });
</script>

{#if hasMore}
    <div bind:this={sentinel} class="flex justify-center pb-24 pt-2">
        <div class="size-6 animate-spin rounded-full border-2 border-line border-t-primary"></div>
    </div>
{/if}
