<script lang="ts">
    import {useScrollContainer} from 'svelte-navigation';
    import type {AssetSummary} from '#lib/api/bindings/AssetSummary';
    import {justify} from '#lib/justify';
    import {mosaicBackground} from '#lib/mosaic';
    import {i18n} from '#lib/state/i18n.svelte';
    import AssetThumb from './AssetThumb.svelte';
    import Sheet from './Sheet.svelte';
    import {grid} from './AssetGrid.i18n';
    import {Calendar, ChevronUp} from '@lucide/svelte';

    let {
        items,
        columns = 3,
        selectionMode = false,
        selected = [],
        onOpen,
        onToggle,
        onColumns,
    }: {
        items: AssetSummary[];
        columns?: number;
        selectionMode?: boolean;
        selected?: string[];
        onOpen: (asset: AssetSummary) => void;
        onToggle?: (asset: AssetSummary) => void;
        onColumns?: (columns: number) => void;
    } = $props();

    type Group = {key: string; label: string; items: AssetSummary[]};

    const t = $derived(i18n.pick(grid));

    const formatters = $derived({
        day: new Intl.DateTimeFormat(i18n.tag, {
            year: 'numeric',
            month: 'long',
            day: 'numeric',
            weekday: 'short',
        }),
        month: new Intl.DateTimeFormat(i18n.tag, {year: 'numeric', month: 'long'}),
    });

    function dayKey(ts: number): string {
        const d = new Date(ts);
        return `${d.getFullYear()}-${d.getMonth()}-${d.getDate()}`;
    }

    function dayLabel(ts: number): string {
        const now = new Date();
        const d = new Date(ts);
        const sameDay =
            now.getFullYear() === d.getFullYear() && now.getMonth() === d.getMonth() && now.getDate() === d.getDate();
        const yesterday = new Date(now.getTime() - 86_400_000);
        const isYesterday =
            yesterday.getFullYear() === d.getFullYear() &&
            yesterday.getMonth() === d.getMonth() &&
            yesterday.getDate() === d.getDate();
        if (sameDay) return t.today;
        if (isYesterday) return t.yesterday;
        return formatters.day.format(d);
    }

    let groups = $derived.by(() => {
        const map = new Map<string, Group>();
        for (const asset of items) {
            const key = dayKey(asset.takenAt);
            const group = map.get(key);
            if (group) group.items.push(asset);
            else map.set(key, {key, label: dayLabel(asset.takenAt), items: [asset]});
        }
        return [...map.values()];
    });

    /** Gap between tiles, matching the reference grid. */
    const SPACING = 2;

    let gridWidth = $state(0);

    /**
     * Rows share a height instead of tiles being cropped to squares. The pinch
     * gesture tunes it through `columns`: fewer columns means taller rows.
     */
    const rowHeight = $derived(gridWidth > 0 ? Math.min(235, Math.max(96, gridWidth / Math.max(2, columns))) : 120);

    let sections = $derived.by(() =>
        groups.map((group) => {
            const ratios = group.items.map((asset) => (asset.width && asset.height ? asset.width / asset.height : 1));
            const layout = justify(ratios, gridWidth, rowHeight, SPACING);
            return {group, boxes: layout.boxes, height: layout.height};
        }),
    );

    type Month = {key: string; label: string; count: number; day: string};

    let scrolled = $state(false);
    let monthSheet = $state(false);

    const scroll = useScrollContainer();

    function monthKey(ts: number): string {
        const d = new Date(ts);
        return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}`;
    }

    // One entry per calendar month, pointing at the first day section.
    let months = $derived.by(() => {
        const map = new Map<string, Month>();
        for (const group of groups) {
            const first = group.items[0];
            if (!first) continue;
            const key = monthKey(first.takenAt);
            const existing = map.get(key);
            if (existing) existing.count += group.items.length;
            else
                map.set(key, {
                    key,
                    label: formatters.month.format(new Date(first.takenAt)),
                    count: group.items.length,
                    day: group.key,
                });
        }
        return [...map.values()];
    });

    function jump(month: Month) {
        monthSheet = false;
        document.getElementById(`day-${month.day}`)?.scrollIntoView({behavior: 'smooth'});
    }

    $effect(() => {
        const element = scroll();
        if (!element) return;
        const onScroll = () => {
            scrolled = element.scrollTop > 1200;
        };
        element.addEventListener('scroll', onScroll, {passive: true});
        onScroll();
        return () => element.removeEventListener('scroll', onScroll);
    });

    function toTop() {
        scroll()?.scrollTo({top: 0, behavior: 'smooth'});
    }

    function pinch(node: HTMLElement, initial: {columns: number; onChange?: (n: number) => void}) {
        let params = initial;
        let startDistance = 0;
        let startColumns = params.columns;
        const distance = (touches: TouchList) =>
            Math.hypot(touches[0]!.clientX - touches[1]!.clientX, touches[0]!.clientY - touches[1]!.clientY);
        const onStart = (event: TouchEvent) => {
            if (event.touches.length !== 2) return;
            startDistance = distance(event.touches);
            startColumns = params.columns;
        };
        const onMove = (event: TouchEvent) => {
            if (event.touches.length !== 2 || !startDistance) return;
            event.preventDefault();
            const ratio = distance(event.touches) / startDistance;
            const next = Math.min(6, Math.max(2, Math.round(startColumns * ratio)));
            if (next !== params.columns) {
                params.onChange?.(next);
            }
        };
        const onEnd = () => {
            startDistance = 0;
        };
        node.addEventListener('touchstart', onStart, {passive: true});
        node.addEventListener('touchmove', onMove, {passive: false});
        node.addEventListener('touchend', onEnd);
        return {
            update(next: {columns: number; onChange?: (n: number) => void}) {
                params = next;
            },
            destroy() {
                node.removeEventListener('touchstart', onStart);
                node.removeEventListener('touchmove', onMove);
                node.removeEventListener('touchend', onEnd);
            },
        };
    }
</script>

<div use:pinch={{columns, onChange: onColumns}} bind:clientWidth={gridWidth} class="pb-24">
    {#each sections as section (section.group.key)}
        <section id="day-{section.group.key}">
            <h2 class="pb-2 pt-6 text-xs font-medium text-muted md:text-sm">
                {section.group.label}
            </h2>
            <div class="relative" style:height="{section.height}px">
                {#each section.boxes as box (section.group.items[box.index]!.id)}
                    {@const asset = section.group.items[box.index]!}
                    <div
                        class="absolute"
                        style:left="{box.left}px"
                        style:top="{box.top}px"
                        style:width="{box.width}px"
                        style:height="{box.height}px"
                    >
                        <AssetThumb
                            source={{kind: 'thumb', assetId: asset.id}}
                            name={asset.name}
                            video={asset.mediaType === 'video'}
                            durationMs={asset.durationMs}
                            favorite={asset.favorite}
                            placeholder={mosaicBackground(asset.thumbHash)}
                            fill
                            {selectionMode}
                            selected={selected.includes(asset.id)}
                            onOpen={() => onOpen(asset)}
                            onToggle={onToggle ? () => onToggle(asset) : undefined}
                        />
                    </div>
                {/each}
            </div>
        </section>
    {/each}
</div>

{#if !selectionMode && months.length > 1}
    <button
        type="button"
        onclick={() => (monthSheet = true)}
        class="fixed bottom-[calc(7.4rem+var(--safe-bottom))] right-3 z-20 grid size-10 place-items-center rounded-full bg-light/90 text-dark shadow-lg backdrop-blur"
        aria-label={t.jumpMonth}
    >
        <Calendar size={19} />
    </button>
{/if}

{#if scrolled && !selectionMode}
    <button
        type="button"
        onclick={toTop}
        class="fixed bottom-[calc(4.2rem+var(--safe-bottom))] right-3 z-20 grid size-10 place-items-center rounded-full bg-light/90 text-dark shadow-lg backdrop-blur"
        aria-label={t.backToTop}
    >
        <ChevronUp size={20} />
    </button>
{/if}

<Sheet bind:open={monthSheet} label={t.jumpToMonth} panelClass="max-h-[60vh] max-w-md">
    <div class="flex min-h-0 flex-1 flex-col p-3">
        <p class="px-2 pb-2 text-xs font-medium text-muted">{t.jumpToMonth}</p>
        <div class="min-h-0 flex-1 overflow-y-auto">
            {#each months as month (month.key)}
                <button
                    onclick={() => jump(month)}
                    class="flex w-full items-center justify-between rounded-xl px-3 py-2.5 text-left text-sm hover:bg-gray"
                >
                    <span>{month.label}</span>
                    <span class="text-[11px] text-muted">{month.count}</span>
                </button>
            {/each}
        </div>
    </div>
</Sheet>
