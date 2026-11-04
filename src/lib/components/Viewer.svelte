<script lang="ts">
    import {api} from '#lib/api/client';
    import type {AssetDetail} from '#lib/api/bindings/AssetDetail';
    import type {AssetSummary} from '#lib/api/bindings/AssetSummary';
    import {mediaSource} from '#lib/api/media';
    import {app} from '#lib/state/app.svelte';
    import AssetInfoSheet from './AssetInfoSheet.svelte';
    import Icon from './Icon.svelte';
    import MediaImage from './MediaImage.svelte';

    let {
        items,
        index = $bindable(0),
        onclose,
    }: {
        items: AssetSummary[];
        index?: number;
        onclose: () => void;
    } = $props();

    let scale = $state(1);
    let translateX = $state(0);
    let translateY = $state(0);
    let src = $state('');
    let videoSrc = $state('');
    let detail = $state<AssetDetail | null>(null);
    let showInfo = $state(false);
    let swipeX = $state(0);
    let favoriteBusy = $state(false);

    let current = $derived(items[index]);
    let isVideo = $derived(current?.mediaType === 'video');

    $effect(() => {
        const asset = items[index];
        if (!asset) return;
        scale = 1;
        translateX = 0;
        translateY = 0;
        detail = null;
        showInfo = false;
        src = '';
        videoSrc = '';
        // Every address comes from the backend: it alone knows whether this
        // platform plays a custom scheme or needs a signed server URL.
        const full = asset.mediaType === 'video' ? 'video' : 'original';
        let live = true;
        void Promise.all([
            mediaSource({kind: 'thumb', assetId: asset.id}),
            mediaSource({kind: full, assetId: asset.id}),
        ]).then(([thumb, resolved]) => {
            if (!live || items[index]?.id !== asset.id) return;
            src = thumb;
            videoSrc = resolved;
            if (asset.mediaType === 'image') {
                // The still replaces the thumbnail only once it decodes, so a slow
                // download never blanks a frame that is already on screen.
                const image = new Image();
                image.onload = () => {
                    if (live && items[index]?.id === asset.id) src = resolved;
                };
                image.src = resolved;
            }
        });
        void api
            .assetDetail({assetId: asset.id})
            .then((value) => {
                if (items[index]?.id === asset.id) detail = value;
            })
            .catch(() => {});
        return () => {
            live = false;
        };
    });

    $effect(() => {
        const previous = document.body.style.overflow;
        document.body.style.overflow = 'hidden';
        return () => {
            document.body.style.overflow = previous;
        };
    });

    $effect(() => {
        const onKey = (event: KeyboardEvent) => {
            if (event.key === 'Escape') onclose();
            if (event.key === 'ArrowRight') next();
            if (event.key === 'ArrowLeft') previous();
            if (event.key === 'f') void toggleFavorite();
        };
        window.addEventListener('keydown', onKey);
        return () => window.removeEventListener('keydown', onKey);
    });

    let touchStartX = 0;
    let pinching = 0;
    let startScale = 1;
    let moved = false;

    function distance(event: TouchEvent): number {
        return Math.hypot(
            event.touches[0]!.clientX - event.touches[1]!.clientX,
            event.touches[0]!.clientY - event.touches[1]!.clientY,
        );
    }

    function onTouchStart(event: TouchEvent) {
        if (event.touches.length === 2) {
            pinching = distance(event);
            startScale = scale;
        } else if (event.touches.length === 1) {
            touchStartX = event.touches[0]!.clientX;
            moved = false;
        }
    }

    function onTouchMove(event: TouchEvent) {
        if (event.touches.length === 2 && pinching > 0) {
            event.preventDefault();
            scale = Math.min(5, Math.max(1, startScale * (distance(event) / pinching)));
            if (scale === 1) {
                translateX = 0;
                translateY = 0;
            }
            return;
        }
        if (event.touches.length === 1 && scale === 1) {
            const dx = event.touches[0]!.clientX - touchStartX;
            if (Math.abs(dx) > 8) moved = true;
            swipeX = dx;
        }
    }

    function onTouchEnd() {
        if (scale > 1) return;
        if (moved && Math.abs(swipeX) > 64) {
            if (swipeX < 0) next();
            else previous();
        }
        swipeX = 0;
        moved = false;
    }

    function next() {
        if (index < items.length - 1) index += 1;
    }

    function previous() {
        if (index > 0) index -= 1;
    }

    function toggleZoom() {
        scale = scale > 1 ? 1 : 2.4;
        if (scale === 1) {
            translateX = 0;
            translateY = 0;
        }
    }

    async function toggleFavorite() {
        const asset = items[index];
        if (!asset || favoriteBusy) return;
        favoriteBusy = true;
        try {
            const updated = await api.setFavorite({
                assetId: asset.id,
                value: !asset.favorite,
            });
            items[index] = updated;
            if (detail) detail = {...detail, asset: updated};
        } catch (error) {
            app.showToast(app.describeError(error), 'error');
        } finally {
            favoriteBusy = false;
        }
    }

    async function download() {
        const asset = items[index];
        if (!asset) return;
        try {
            const result = await api.exportAssets({assetIds: [asset.id]});
            app.showToast(`已导出 ${result.exported} 项到 ${result.directory}`, 'success');
        } catch (error) {
            app.showToast(app.describeError(error), 'error');
        }
    }
</script>

<div class="fixed inset-0 z-50 flex flex-col bg-black text-white">
    <div class="grid grid-cols-[1fr_auto_1fr] items-center gap-2 px-2 pt-[var(--safe-top)]">
        <button
            onclick={onclose}
            class="grid size-11 place-items-center justify-self-start rounded-full hover:bg-white/10"
            aria-label="关闭"
        >
            <Icon name="close" size={24} />
        </button>
        <div class="min-w-0 max-w-[calc(100%-20.5rem)] text-center">
            <p class="truncate text-sm font-medium">{current?.name ?? ''}</p>
            <p class="text-[11px] text-white/60">{index + 1} / {items.length}</p>
        </div>
        <div class="flex items-center justify-self-end gap-2">
            <button
                onclick={toggleFavorite}
                class="grid size-11 place-items-center rounded-full hover:bg-white/10"
                aria-label="收藏"
            >
                <Icon
                    name={current?.favorite ? 'heart-filled' : 'heart'}
                    size={23}
                    class={current?.favorite ? 'text-rose-500' : ''}
                />
            </button>
            <button
                onclick={download}
                class="grid size-11 place-items-center rounded-full hover:bg-white/10"
                aria-label="下载"
            >
                <Icon name="download" size={22} />
            </button>
            <button
                onclick={() => (showInfo = !showInfo)}
                class="grid size-11 place-items-center rounded-full hover:bg-white/10"
                aria-label="信息"
            >
                <Icon name="info" size={22} />
            </button>
        </div>
    </div>

    <div
        class="relative flex min-h-0 flex-1 items-center justify-center overflow-hidden"
        role="group"
        aria-label="媒体查看器"
        ontouchstart={onTouchStart}
        ontouchmove={onTouchMove}
        ontouchend={onTouchEnd}
        ondblclick={toggleZoom}
    >
        {#if current}
            {#if isVideo}
                {#if videoSrc}
                    <!-- svelte-ignore a11y_media_has_caption -->
                    <video src={videoSrc} controls autoplay playsinline class="max-h-full max-w-full"></video>
                {/if}
            {:else if src}
                <img
                    {src}
                    alt={current.name}
                    class="max-h-full max-w-full select-none object-contain transition-transform duration-200 ease-out"
                    style:transform="translate({translateX + swipeX}px, {translateY}px) scale({scale})"
                    draggable="false"
                />
            {/if}
        {/if}

        <button
            onclick={previous}
            disabled={index === 0}
            class="absolute left-1 top-1/2 grid size-10 -translate-y-1/2 place-items-center rounded-full bg-black/30 text-white disabled:opacity-0"
            aria-label="上一张"
        >
            <Icon name="back" size={24} />
        </button>
        <button
            onclick={next}
            disabled={index === items.length - 1}
            class="absolute right-1 top-1/2 grid size-10 -translate-y-1/2 place-items-center rounded-full bg-black/30 text-white disabled:opacity-0"
            aria-label="下一张"
        >
            <Icon name="forward" size={24} />
        </button>
    </div>

    <div class="pb-[var(--safe-bottom)]">
        <div class="flex gap-1 overflow-x-auto px-2 py-2" style="-webkit-overflow-scrolling: touch">
            {#each items as item, i (item.id)}
                <button
                    onclick={() => (index = i)}
                    class="relative size-12 shrink-0 overflow-hidden rounded-md {i === index
                        ? 'ring-2 ring-white'
                        : 'opacity-60'}"
                    aria-label={item.name}
                >
                    <MediaImage source={{kind: 'thumb', assetId: item.id}} alt="" class="size-full object-cover" />
                    {#if item.mediaType === 'video'}
                        <span class="absolute inset-0 grid place-items-center bg-black/20">
                            <Icon name="play" size={14} />
                        </span>
                    {/if}
                </button>
            {/each}
        </div>
    </div>

    <AssetInfoSheet bind:open={showInfo} asset={current} {detail} />
</div>
