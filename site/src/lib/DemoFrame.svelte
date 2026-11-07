<script lang="ts">
    import {i18n} from './i18n.svelte';
    import {text} from './text';
    import {theme} from './theme.svelte';

    /** Outer size of the `devices.css` iPhone frame this page is built around. */
    const FRAME_W = 428;
    const FRAME_H = 868;
    /** Tall windows get a bigger phone; beyond that the frame looks blurry. */
    const MAX_SCALE = 1.35;

    /**
     * Dev proxies the demo's own dev server under this same path, so the frame
     * is same-origin in both modes and shares the site's storage.
     */
    const src = `${import.meta.env.BASE_URL}app/`;

    let {reload}: {reload: number} = $props();

    let fit: HTMLElement;
    let frame = $state<HTMLIFrameElement>();
    let scale = $state(1);

    $effect(() => {
        const observer = new ResizeObserver(([entry]) => {
            const {width, height} = entry.contentRect;
            scale = Math.min(MAX_SCALE, width / FRAME_W, height / FRAME_H);
        });
        observer.observe(fit);
        return () => observer.disconnect();
    });

    // The framed app has its own document; repaint it without losing the route.
    $effect(() => {
        const root = frame?.contentDocument?.documentElement;
        if (!root) return;
        root.classList.toggle('dark', theme.value === 'dark');
        root.classList.toggle('light', theme.value === 'light');
    });

    const t = $derived(i18n.pick(text));
</script>

<div class="fit" bind:this={fit}>
    <div class="holder" style:width="{FRAME_W * scale}px" style:height="{FRAME_H * scale}px">
        <!-- The shell itself comes from devices.css; only the screen content is ours. -->
        <div
            class="device device-iphone-14-pro device-black"
            style:transform="scale({scale})"
            style:transform-origin="top left"
        >
            <div class="device-frame">
                <div class="device-screen">
                    <!-- The app reads the locale once at boot, so the site's switch reloads it. -->
                    {#key reload}
                        <iframe bind:this={frame} {src} title={t.previewTitle}></iframe>
                    {/key}
                </div>
            </div>
            <div class="device-stripe"></div>
            <div class="device-header"></div>
            <div class="device-sensors"></div>
            <div class="device-btns"></div>
            <div class="device-power"></div>
        </div>
    </div>
</div>

<style>
    .fit {
        display: flex;
        flex: 1;
        align-items: center;
        justify-content: center;
        overflow: hidden;
        min-width: 0;
        min-height: 0;
    }

    .holder {
        position: relative;
    }

    .device-screen {
        display: flex;
        flex-direction: column;
        /* The 49px corners of the shell, in the 390x830 screen coordinates. */
        --screen-mask: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='390' height='830'%3E%3Crect width='390' height='830' rx='49' fill='white'/%3E%3C/svg%3E");
    }

    .device-screen iframe {
        display: block;
        flex: 1;
        width: 100%;
        border: 0;
        background: transparent;
        /* The 49px corners are cut on the frame itself, so its square edges never
           reach the shell. The SVG has to stay percent encoded: Gecko rejects the
           raw utf8 data URI and then paints the iframe square over the bezel. */
        border-radius: 49px;
        clip-path: inset(0 round 49px);
        -webkit-mask-image: var(--screen-mask);
        mask-image: var(--screen-mask);
        -webkit-mask-size: 100% 100%;
        mask-size: 100% 100%;
    }
</style>
