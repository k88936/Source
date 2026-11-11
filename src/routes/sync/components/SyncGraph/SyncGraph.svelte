<script lang="ts">
    import type {AlbumMetadata} from '#lib/api/bindings/AlbumMetadata';
    import type {LocalAlbumMetadata} from '#lib/api/bindings/LocalAlbumMetadata';
    import {app} from '#lib/app/app.svelte';

    import type {Group, Mapping} from '../../sync.svelte';
    import AddMappingCard from '../AddMappingCard/AddMappingCard.svelte';
    import AlbumNode from '../AlbumNode/AlbumNode.svelte';
    import MappingCard from '../MappingCard/MappingCard.svelte';
    import {curve, GLINT_WIDTH, glintsOf, type Route} from './graph';

    let {
        groups,
        orphans,
        onAlbum,
        onLocal,
        onAdd,
    }: {
        groups: Group[];
        orphans: Mapping[];
        onAlbum: (album: AlbumMetadata) => void;
        onLocal: (local: LocalAlbumMetadata) => void;
        onAdd: (album: AlbumMetadata) => void;
    } = $props();

    let surface = $state<HTMLDivElement | null>(null);
    let frame = $state({width: 0, height: 0});
    let routes = $state<Route[]>([]);

    /// The rows change when a mapping is added or removed; the measuring only
    /// has to run then, and whenever the browser lays the rows out differently.
    const shape = $derived(
        groups.map((group) => `${group.album.id}:${group.mappings.map((m) => m.local.id).join(',')}`).join('|') +
            `|${orphans.length}`,
    );

    $effect(() => {
        void shape;
        const root = surface;
        if (!root) return;

        const measure = () => {
            const base = root.getBoundingClientRect();
            const sinks = new Map<string, DOMRect>();
            for (const node of root.querySelectorAll<HTMLElement>('[data-sink]')) {
                sinks.set(node.dataset.sink ?? '', node.getBoundingClientRect());
            }
            const next: Route[] = [];
            for (const node of root.querySelectorAll<HTMLElement>('[data-edge]')) {
                const sink = sinks.get(node.dataset.album ?? '');
                if (!sink) continue;
                const from = node.getBoundingClientRect();
                next.push(
                    curve(
                        node.dataset.edge ?? '',
                        from.right - base.left,
                        from.top + from.height / 2 - base.top,
                        sink.left - base.left,
                        sink.top + sink.height / 2 - base.top,
                    ),
                );
            }
            frame = {width: base.width, height: base.height};
            routes = next;
        };

        measure();
        const observer = new ResizeObserver(measure);
        observer.observe(root);
        for (const node of root.querySelectorAll('[data-edge], [data-sink]')) {
            observer.observe(node);
        }
        return () => observer.disconnect();
    });

    /// Only bytes on the wire animate: a folder that is not moving bytes has no
    /// entry, and its pipe stays a still wire.
    const speeds = $derived(
        new Map(
            groups.flatMap((group) =>
                group.mappings.map((mapping) => [mapping.local.id, mapping.tally.speed] as const),
            ),
        ),
    );

    const glints = $derived(glintsOf(routes, (id) => speeds.get(id) ?? 0));
</script>

<div bind:this={surface} class="relative mt-7">
    {#if routes.length > 0}
        <svg
            class="pointer-events-none absolute inset-0 z-0 overflow-visible"
            width={frame.width}
            height={frame.height}
            viewBox={`0 0 ${frame.width} ${frame.height}`}
            class:is-paused={app.jobsPaused}
            aria-hidden="true"
        >
            <defs>
                <linearGradient id="sync-glint-fade" x1="0" y1="0" x2="1" y2="0">
                    <stop offset="0%" stop-color="white" stop-opacity="0" />
                    <stop offset="35%" stop-color="white" stop-opacity="1" />
                    <stop offset="65%" stop-color="white" stop-opacity="1" />
                    <stop offset="100%" stop-color="white" stop-opacity="0" />
                </linearGradient>
                {#each glints as glint (glint.route.id)}
                    <mask
                        id="sync-glint-{glint.route.id}"
                        maskUnits="userSpaceOnUse"
                        x={glint.route.x1 - GLINT_WIDTH}
                        y={glint.top}
                        width={glint.travel + GLINT_WIDTH}
                        height={glint.height}
                    >
                        <rect
                            class="glint"
                            x={glint.route.x1 - GLINT_WIDTH}
                            y={glint.top}
                            width={GLINT_WIDTH}
                            height={glint.height}
                            fill="url(#sync-glint-fade)"
                            style="--glint-time: {glint.time}; --travel: {glint.travel}px"
                        />
                    </mask>
                {/each}
            </defs>
            {#each routes as route (route.id)}
                <path class="edge" d={route.d} fill="none" />
            {/each}
            {#each glints as glint (glint.route.id)}
                <g mask="url(#sync-glint-{glint.route.id})">
                    <path class="pipe-glow" d={glint.route.d} fill="none" />
                    <path class="pipe-core" d={glint.route.d} fill="none" />
                </g>
            {/each}
        </svg>
    {/if}
    <div class="relative z-10 flex flex-col gap-6">
        {#each groups as group (group.album.id)}
            <div class="grid grid-cols-[minmax(0,1fr)_minmax(0,1fr)] items-center gap-x-10">
                <div class="flex min-w-0 flex-col items-stretch gap-2">
                    {#each group.mappings as mapping (mapping.local.id)}
                        <MappingCard {mapping} onOpen={() => onLocal(mapping.local)} />
                    {/each}
                    <AddMappingCard albumId={group.album.id} onOpen={() => onAdd(group.album)} />
                </div>
                <AlbumNode {group} onOpen={() => onAlbum(group.album)} />
            </div>
        {/each}
        {#if orphans.length > 0}
            <div class="grid grid-cols-[minmax(0,1fr)_minmax(0,1fr)] gap-x-10">
                <div class="flex min-w-0 flex-col items-stretch gap-2">
                    {#each orphans as mapping (mapping.local.id)}
                        <MappingCard {mapping} onOpen={() => onLocal(mapping.local)} />
                    {/each}
                </div>
            </div>
        {/if}
    </div>
</div>

<style>
    .edge {
        stroke: var(--color-muted);
        stroke-width: 1.5;
        stroke-linecap: round;
        stroke-linejoin: round;
        opacity: 0.35;
    }

    /* A pipe is always the same thin wire. What moves is a window of light
       inside it, and both strokes below share that one mask: a wide soft one to
       glow against the wall and a bright core to read as liquid, so the light
       looks like it is filling the tube rather than travelling over it. */
    .pipe-glow {
        stroke: var(--color-primary);
        stroke-width: 6;
        stroke-linecap: round;
        opacity: 0.18;
        filter: blur(2px);
    }

    .pipe-core {
        stroke: var(--color-primary);
        stroke-width: 1.2;
        stroke-linecap: round;
        opacity: 0.85;
    }

    .glint {
        animation: sync-glint var(--glint-time, 2s) linear infinite;
    }

    .is-paused .glint {
        animation-play-state: paused;
    }

    @keyframes sync-glint {
        from {
            transform: translateX(0);
        }
        to {
            transform: translateX(var(--travel, 0px));
        }
    }

    @media (prefers-reduced-motion: reduce) {
        .glint {
            animation: none;
        }
    }
</style>
