<script lang="ts">
    import {onDestroy} from 'svelte';
    import type {MediaSourceParams} from '#lib/api/bindings/MediaSourceParams';
    import {mediaSource} from '#lib/api/media';
    import {app} from '#lib/app/app.svelte';

    import {Check, Heart, Play} from '@lucide/svelte';

    const LONG_PRESS_MS = 500;
    const PRESS_SLOP = 10;

    let {
        source,
        name,
        video = false,
        durationMs = null,
        favorite = false,
        placeholder = null,
        badge = '',
        selectionMode = false,
        selected = false,
        /** Let the caller size the tile (justified rows) instead of forcing a square. */
        fill = false,
        onOpen,
        onToggle,
    }: {
        /** Resolved only once the tile scrolls into view, so a long grid asks the
         * backend for addresses it is about to paint and nothing more. */
        source?: MediaSourceParams;
        name: string;
        video?: boolean;
        durationMs?: number | null;
        favorite?: boolean;
        /** Blur-up mosaic (`mosaicBackground(...)`) shown until the JPEG arrives. */
        placeholder?: string | null;
        /** Small label in the corner, e.g. a file size. */
        badge?: string;
        selectionMode?: boolean;
        selected?: boolean;
        fill?: boolean;
        onOpen?: () => void;
        onToggle?: () => void;
    } = $props();

    let host: HTMLButtonElement | undefined = $state();
    let loaded = $state(false);
    let visible = $state(false);
    let src = $state('');

    $effect(() => {
        if (!host || visible) return;
        const observer = new IntersectionObserver(
            (entries) => {
                if (entries.some((entry) => entry.isIntersecting)) visible = true;
            },
            {rootMargin: '600px 0px'},
        );
        observer.observe(host);
        return () => observer.disconnect();
    });

    // The address is minted for the account the page shows, but only a live
    // client can honour it: a tile painted during the start-up check is
    // reloaded when the answer arrives, because the address does not change.
    $effect(() => {
        void app.account?.user.id;
        void app.setup;
        if (!visible || !source) return;
        let live = true;
        void mediaSource(source).then((url) => {
            if (live) src = url;
        });
        return () => {
            live = false;
        };
    });

    function formatDuration(ms: number | null): string {
        if (!ms || ms <= 0) return '';
        const total = Math.round(ms / 1000);
        return `${Math.floor(total / 60)}:${String(total % 60).padStart(2, '0')}`;
    }

    let pressTimer: ReturnType<typeof setTimeout> | undefined;
    let pressOrigin = {x: 0, y: 0};
    /** A long press already picked this tile; swallow the click that follows. */
    let held = false;

    function startPress(event: PointerEvent) {
        if (!event.isPrimary || selectionMode || !onToggle || event.button !== 0) return;
        cancelPress();
        held = false;
        pressOrigin = {x: event.clientX, y: event.clientY};
        pressTimer = setTimeout(() => {
            pressTimer = undefined;
            held = true;
            onToggle?.();
        }, LONG_PRESS_MS);
    }

    function cancelPress() {
        if (pressTimer === undefined) return;
        clearTimeout(pressTimer);
        pressTimer = undefined;
    }

    function trackPress(event: PointerEvent) {
        if (pressTimer === undefined) return;
        if (Math.hypot(event.clientX - pressOrigin.x, event.clientY - pressOrigin.y) > PRESS_SLOP) cancelPress();
    }

    function click() {
        if (held) {
            held = false;
            return;
        }
        if (selectionMode) onToggle?.();
        else onOpen?.();
    }

    onDestroy(cancelPress);
</script>

<button
    bind:this={host}
    type="button"
    class="group relative block overflow-hidden bg-gray text-left outline-none transition-[transform,border-radius] duration-200 {fill
        ? 'size-full'
        : 'aspect-square w-full'} {selected ? 'scale-90 rounded-lg' : ''}"
    onclick={click}
    onpointerdown={startPress}
    onpointermove={trackPress}
    onpointerup={cancelPress}
    onpointercancel={cancelPress}
    onpointerleave={cancelPress}
    oncontextmenu={(event) => event.preventDefault()}
    aria-label={name}
>
    {#if visible && src && !loaded}
        {#if placeholder}
            <!-- 4x4 mosaic from the thumbnail, blurred: no grey flash on first paint. -->
            <span
                class="pointer-events-none absolute -inset-2 scale-110 blur-lg"
                style:background-image={placeholder}
                style:background-size="cover"
            ></span>
        {:else}
            <span class="pointer-events-none absolute inset-0 animate-pulse bg-muted/25"></span>
        {/if}
    {/if}
    {#if visible && src}
        {#key app.setup}
            <img
                {src}
                alt={name}
                loading="lazy"
                decoding="async"
                draggable="false"
                onload={() => (loaded = true)}
                class="relative size-full object-cover transition-opacity duration-300 {loaded
                    ? 'opacity-100'
                    : 'opacity-0'}"
            />
        {/key}
    {/if}
    <!-- Nothing is layered over the photo: labels carry their own shadow instead. -->
    <div class="pointer-events-none absolute inset-x-0 bottom-0 flex items-end justify-between gap-2 p-2">
        <span class="flex min-w-0 items-center gap-1.5">
            {#if favorite}
                <Heart size={22} class="icon-white-drop-shadow text-white" fill="currentColor" />
            {/if}
        </span>
        <span class="flex shrink-0 items-center gap-1">
            {#if video}
                <Play size={16} class="icon-white-drop-shadow text-white" fill="currentColor" />
            {/if}
            {#if durationMs}
                <span class="text-xs font-medium text-white text-white-shadow">
                    {formatDuration(durationMs)}
                </span>
            {:else if badge}
                <span class="text-xs font-medium text-white text-white-shadow">{badge}</span>
            {/if}
        </span>
    </div>
    {#if selectionMode}
        <span
            class="pointer-events-none absolute right-2 top-2 grid size-6 place-items-center rounded-full border-2 text-white transition-colors {selected
                ? 'border-white bg-primary'
                : 'border-white/90 bg-black/15'}"
        >
            {#if selected}<Check size={14} />{/if}
        </span>
    {/if}
</button>
