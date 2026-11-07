<script lang="ts">
    import '../app.css';
    // Importing the plugin publishes `--safe-area-inset-*` on `:root`; it hooks the
    // keyboard events itself, so there is nothing to call.
    import '@saurl/tauri-plugin-safe-area-insets-css-api';
    import {onMount} from 'svelte';
    import {goto} from '$app/navigation';
    import {resolve} from '$app/paths';
    import {page} from '$app/state';
    import {Cloud} from '@lucide/svelte';
    import Sidebar from '#lib/components/Sidebar.svelte';
    import TabBar from '#lib/components/TabBar.svelte';
    import Toast from '#lib/components/Toast.svelte';
    import {listen} from '#lib/api/client';
    import {app} from '#lib/state/app.svelte';
    import {i18n} from '#lib/state/i18n.svelte';
    import {layout} from './layout.i18n';

    const {children} = $props();

    const t = $derived(i18n.pick(layout));

    // The shell (sidebar gutter + tab bar) belongs to the signed-in pages only:
    // the login screen is a standalone card, so it must not reserve the sidebar.
    const shell = $derived(app.ready && !!app.account && page.route.id !== '/welcome');

    onMount(() => {
        void app.init();
        // The adapter is the only place that can tell a refused session from a
        // wrong password, so it reports the loss and the shell reacts; nothing
        // here inspects an error kind.
        let unlisten: (() => void) | undefined;
        void listen('session://lost', () => {
            app.account = null;
            app.showToast(t.sessionExpired, 'error');
            void goto(resolve('/welcome'));
        }).then((remove) => {
            unlisten = remove;
        });
        return () => unlisten?.();
    });

    $effect(() => {
        if (!app.ready) return;
        const route = page.route.id;
        if (!app.account && route !== '/welcome') {
            void goto(resolve('/welcome'));
        } else if (app.account && route === '/welcome') {
            void goto(resolve('/photos'));
        }
    });
</script>

{#if !app.ready}
    <div class="grid min-h-dvh place-items-center bg-light text-dark">
        <div class="flex flex-col items-center gap-4">
            <div class="grid size-16 animate-pulse place-items-center rounded-2xl bg-primary text-on-primary">
                <Cloud size={30} />
            </div>
            <p class="text-sm text-muted">{t.loading}</p>
        </div>
    </div>
{:else}
    <div class="min-h-dvh bg-light text-dark">
        {#if shell}
            <Sidebar />
        {/if}
        <div class={shell ? 'sidebar:ps-60' : ''}>
            {@render children()}
        </div>
    </div>
    {#if shell}
        <TabBar />
    {/if}
    <Toast />
{/if}
