<script lang="ts">
    import {api} from '#lib/api/client';
    import type {AssetDetail} from '#lib/api/bindings/AssetDetail';
    import type {AssetSummary} from '#lib/api/bindings/AssetSummary';
    import {mediaSource} from '#lib/api/media';
    import {common} from '#lib/i18n/common.i18n';
    import {app} from '#lib/state/app.svelte';
    import {i18n} from '#lib/state/i18n.svelte';
    import AssetInfoSheet from './AssetInfoSheet.svelte';
    import {ChevronLeft, ChevronRight, Download, Heart, Info, Play, X} from '@lucide/svelte';
    import MediaImage from './MediaImage.svelte';
    import {viewer} from './Viewer.i18n';

    let {
        items,
        index = $bindable(0),
        onClose,
    }: {
        items: AssetSummary[];
        index?: number;
        onClose: () => void;
    } = $props();

    const t = $derived(i18n.pick(viewer));
    const c = $derived(i18n.pick(common));

    let scale = $state(1);
    let translateX = $state(0);
    let translateY = $state(0);
    let src = $state('');
    let videoSrc = $state('');
    let detail = $state<AssetDetail | null>(null);
    let showInfo = $state(false);
    let swipeX = $state(0);
    let gesturing = $state(false);
    let imageElement = $state<HTMLImageElement | null>(null);
    let favoriteBusy = $state(false);

    let current = $derived(items[index]);
    let isVideo = $derived(current?.mediaType === 'video');

    $effect(() => {
        const asset = items[index];
        if (!asset) return;
        scale = 1;
        translateX = 0;
        translateY = 0;
        swipeX = 0;
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
            if (event.key === 'Escape') onClose();
            if (event.key === 'ArrowRight') next();
            if (event.key === 'ArrowLeft') previous();
            if (event.key === 'f') void toggleFavorite();
        };
        window.addEventListener('keydown', onKey);
        return () => window.removeEventListener('keydown', onKey);
    });

    function next() {
        if (index < items.length - 1) index += 1;
    }

    function previous() {
        if (index > 0) index -= 1;
    }

    const MAX_SCALE = 5;
    const TAP_SLOP = 8;
    const TAP_DELAY = 300;
    const SWIPE_THRESHOLD = 64;

    type Point = {x: number; y: number};

    type Pan = {
        pointer: number;
        originX: number;
        originY: number;
        translateX: number;
        translateY: number;
        paging: boolean;
        moved: boolean;
    };

    type Pinch = {
        distance: number;
        x: number;
        y: number;
        scale: number;
        translateX: number;
        translateY: number;
    };

    function clamp(value: number, limit: number): number {
        return Math.min(limit, Math.max(-limit, value));
    }

    function gestures(node: HTMLElement) {
        const points = new Map<number, Point>();
        let center: Point = {x: 0, y: 0};
        let pan: Pan | null = null;
        let pinch: Pinch | null = null;
        let lastTap = 0;
        let lastTapAt: Point = {x: 0, y: 0};

        // How far the scaled image can travel before an edge leaves the frame.
        function limits(atScale: number): Point {
            const image = imageElement;
            if (!image) return {x: 0, y: 0};
            return {
                x: Math.max(0, (image.clientWidth * atScale - node.clientWidth) / 2),
                y: Math.max(0, (image.clientHeight * atScale - node.clientHeight) / 2),
            };
        }

        function settle() {
            const limit = limits(scale);
            translateX = clamp(translateX, limit.x);
            translateY = clamp(translateY, limit.y);
        }

        function zoomAt(point: Point) {
            if (scale > 1) {
                scale = 1;
                translateX = 0;
                translateY = 0;
                return;
            }
            const next = 2.4;
            const limit = limits(next);
            // The tapped point stays under the finger while the image grows around it.
            translateX = clamp(-(next - 1) * point.x, limit.x);
            translateY = clamp(-(next - 1) * point.y, limit.y);
            scale = next;
        }

        function onPointerDown(event: PointerEvent) {
            if (isVideo) return;
            if (event.pointerType === 'mouse' && event.button !== 0) return;
            if ((event.target as Element).closest('button')) return;

            const rect = node.getBoundingClientRect();
            center = {x: rect.left + rect.width / 2, y: rect.top + rect.height / 2};
            points.set(event.pointerId, {x: event.clientX, y: event.clientY});
            gesturing = true;
            if (event.pointerType === 'mouse') node.setPointerCapture(event.pointerId);

            if (points.size === 1) {
                pan = {
                    pointer: event.pointerId,
                    originX: event.clientX - center.x,
                    originY: event.clientY - center.y,
                    translateX,
                    translateY,
                    paging: scale === 1,
                    moved: false,
                };
                pinch = null;
            } else if (points.size === 2) {
                const first = [...points.values()][0]!;
                const second = [...points.values()][1]!;
                swipeX = 0;
                pinch = {
                    distance: Math.hypot(first.x - second.x, first.y - second.y),
                    x: (first.x + second.x) / 2 - center.x,
                    y: (first.y + second.y) / 2 - center.y,
                    scale,
                    translateX,
                    translateY,
                };
                pan = null;
            }
        }

        function onPointerMove(event: PointerEvent) {
            if (!points.has(event.pointerId)) return;
            points.set(event.pointerId, {x: event.clientX, y: event.clientY});

            if (pinch && points.size >= 2) {
                const first = [...points.values()][0]!;
                const second = [...points.values()][1]!;
                const distance = Math.hypot(first.x - second.x, first.y - second.y);
                const next = Math.min(MAX_SCALE, Math.max(1, (pinch.scale * distance) / pinch.distance));
                const ratio = next / pinch.scale;
                const limit = limits(next);
                // Anchor the content under the starting midpoint, which also pans with two fingers.
                translateX = clamp((first.x + second.x) / 2 - center.x - (pinch.x - pinch.translateX) * ratio, limit.x);
                translateY = clamp((first.y + second.y) / 2 - center.y - (pinch.y - pinch.translateY) * ratio, limit.y);
                scale = next;
                if (next === 1) {
                    translateX = 0;
                    translateY = 0;
                }
                return;
            }

            if (!pan || pan.pointer !== event.pointerId) return;
            const dx = event.clientX - center.x - pan.originX;
            const dy = event.clientY - center.y - pan.originY;
            if (Math.abs(dx) > TAP_SLOP || Math.abs(dy) > TAP_SLOP) pan.moved = true;
            if (pan.paging) {
                swipeX = dx;
                return;
            }
            const limit = limits(scale);
            translateX = clamp(pan.translateX + dx, limit.x);
            translateY = clamp(pan.translateY + dy, limit.y);
        }

        function onPointerUp(event: PointerEvent) {
            if (!points.delete(event.pointerId)) return;

            if (pinch) {
                pinch = null;
                const rest = [...points.entries()][0];
                if (rest) {
                    // The finger still down carries on as a pan, so the image does not jump.
                    const [pointer, point] = rest;
                    pan = {
                        pointer,
                        originX: point.x - center.x,
                        originY: point.y - center.y,
                        translateX,
                        translateY,
                        paging: false,
                        moved: true,
                    };
                    return;
                }
                gesturing = false;
                settle();
                return;
            }

            if (!pan || pan.pointer !== event.pointerId) return;
            const {paging, moved} = pan;
            pan = null;
            gesturing = false;

            if (paging) {
                const swipe = swipeX;
                swipeX = 0;
                if (moved && Math.abs(swipe) > SWIPE_THRESHOLD) {
                    if (swipe < 0) next();
                    else previous();
                    return;
                }
            }
            if (moved) {
                settle();
                return;
            }

            const point = {x: event.clientX - center.x, y: event.clientY - center.y};
            const now = performance.now();
            const repeated = now - lastTap < TAP_DELAY && Math.hypot(point.x - lastTapAt.x, point.y - lastTapAt.y) < 24;
            lastTap = repeated ? 0 : now;
            lastTapAt = point;
            if (repeated) zoomAt(point);
        }

        function onPointerCancel() {
            points.clear();
            pan = null;
            pinch = null;
            gesturing = false;
            swipeX = 0;
            settle();
        }

        node.addEventListener('pointerdown', onPointerDown);
        node.addEventListener('pointermove', onPointerMove);
        node.addEventListener('pointerup', onPointerUp);
        node.addEventListener('pointercancel', onPointerCancel);
        window.addEventListener('resize', settle);
        return {
            destroy() {
                node.removeEventListener('pointerdown', onPointerDown);
                node.removeEventListener('pointermove', onPointerMove);
                node.removeEventListener('pointerup', onPointerUp);
                node.removeEventListener('pointercancel', onPointerCancel);
                window.removeEventListener('resize', settle);
            },
        };
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
            app.showToast(t.exported(result.exported, result.directory), 'success');
        } catch (error) {
            app.showToast(app.describeError(error), 'error');
        }
    }
</script>

<div class="fixed inset-0 z-50 flex flex-col bg-black text-white">
    <div class="grid grid-cols-[1fr_auto_1fr] items-center gap-2 px-2 pt-[var(--safe-top)]">
        <button
            onclick={onClose}
            class="grid size-11 place-items-center justify-self-start rounded-full hover:bg-white/10"
            aria-label={c.close}
        >
            <X size={24} />
        </button>
        <div class="min-w-0 text-center">
            <p class="truncate text-sm font-medium">{current?.name ?? ''}</p>
            <p class="text-[11px] text-white/60">{index + 1} / {items.length}</p>
        </div>
        <div class="flex items-center justify-self-end gap-2">
            <button
                onclick={toggleFavorite}
                class="grid size-11 place-items-center rounded-full hover:bg-white/10"
                aria-label={t.favorite}
            >
                <Heart
                    size={23}
                    fill={current?.favorite ? 'currentColor' : 'none'}
                    class={current?.favorite ? 'text-rose-500' : ''}
                />
            </button>
            <button
                onclick={download}
                class="grid size-11 place-items-center rounded-full hover:bg-white/10"
                aria-label={c.download}
            >
                <Download size={22} />
            </button>
            <button
                onclick={() => (showInfo = !showInfo)}
                class="grid size-11 place-items-center rounded-full hover:bg-white/10"
                aria-label={t.info}
            >
                <Info size={22} />
            </button>
        </div>
    </div>

    <div
        class="relative flex min-h-0 flex-1 items-center justify-center overflow-hidden {isVideo ? '' : 'touch-none'}"
        role="group"
        aria-label={t.mediaViewer}
        use:gestures
    >
        {#if current}
            {#if isVideo}
                {#if videoSrc}
                    <!-- svelte-ignore a11y_media_has_caption -->
                    <video src={videoSrc} controls autoplay playsinline class="max-h-full max-w-full"></video>
                {/if}
            {:else if src}
                <img
                    bind:this={imageElement}
                    {src}
                    alt={current.name}
                    class="max-h-full max-w-full select-none object-contain {gesturing
                        ? ''
                        : 'transition-transform duration-200 ease-out'}"
                    style:transform="translate({translateX + swipeX}px, {translateY}px) scale({scale})"
                    draggable="false"
                />
            {/if}
        {/if}

        <button
            onclick={previous}
            disabled={index === 0}
            class="absolute left-1 top-1/2 grid size-10 -translate-y-1/2 place-items-center rounded-full bg-black/30 text-white disabled:opacity-0"
            aria-label={t.previous}
        >
            <ChevronLeft size={24} />
        </button>
        <button
            onclick={next}
            disabled={index === items.length - 1}
            class="absolute right-1 top-1/2 grid size-10 -translate-y-1/2 place-items-center rounded-full bg-black/30 text-white disabled:opacity-0"
            aria-label={t.next}
        >
            <ChevronRight size={24} />
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
                            <Play size={14} fill="currentColor" />
                        </span>
                    {/if}
                </button>
            {/each}
        </div>
    </div>

    <AssetInfoSheet bind:open={showInfo} asset={current} {detail} />
</div>
