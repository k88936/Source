<script lang="ts">
    import type {Search} from '#lib/state/search.svelte';
    import {i18n} from '#lib/state/i18n.svelte';
    import {common} from '#lib/i18n/common.i18n';
    import {search as text} from '#lib/i18n/search.i18n';
    import {Heart, Image, Video} from '@lucide/svelte';

    let {search}: {search: Search} = $props();

    const t = $derived(i18n.pick(text));
    const c = $derived(i18n.pick(common));

    const on = 'bg-primary text-on-primary';
    const off = 'bg-gray text-muted hover:text-dark';

    function chipClass(active: boolean): string {
        return `shrink-0 rounded-full px-3.5 py-1.5 text-xs font-medium transition-colors ${active ? on : off}`;
    }
</script>

<div class="mx-auto max-w-3xl px-4 pt-2">
    <div class="flex gap-2 overflow-x-auto pb-1">
        <button onclick={() => search.setMediaType(null)} class={chipClass(search.mediaType === null)}>
            {c.all}
        </button>
        <button onclick={() => search.setMediaType('image')} class={chipClass(search.mediaType === 'image')}>
            <Image size={13} class="mr-1 inline" />{t.image}
        </button>
        <button onclick={() => search.setMediaType('video')} class={chipClass(search.mediaType === 'video')}>
            <Video size={13} class="mr-1 inline" />{t.video}
        </button>
        <button onclick={() => search.toggleFavorite()} class={chipClass(search.favorite)}>
            <Heart size={13} class="mr-1 inline" />{t.favorites}
        </button>
    </div>

    <div class="mt-2 flex items-center gap-2">
        <label class="flex flex-1 items-center gap-1.5 rounded-lg bg-gray px-3 py-2 text-xs text-muted">
            <span>{t.from}</span>
            <input
                value={search.from}
                oninput={(event) => search.setRange(event.currentTarget.value, search.to)}
                type="date"
                class="w-full bg-transparent text-xs text-dark outline-none"
            />
        </label>
        <label class="flex flex-1 items-center gap-1.5 rounded-lg bg-gray px-3 py-2 text-xs text-muted">
            <span>{t.to}</span>
            <input
                value={search.to}
                oninput={(event) => search.setRange(search.from, event.currentTarget.value)}
                type="date"
                class="w-full bg-transparent text-xs text-dark outline-none"
            />
        </label>
    </div>
</div>
