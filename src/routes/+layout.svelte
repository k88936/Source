<script lang="ts">
    import '../app.css';
    import {onMount} from 'svelte';
    import {goto} from '$app/navigation';
    import {page} from '$app/state';
    import {Cloud} from '@lucide/svelte';
    import Sidebar from '#lib/components/Sidebar.svelte';
    import TabBar from '#lib/components/TabBar.svelte';
    import Toast from '#lib/components/Toast.svelte';
    import {app} from '#lib/state/app.svelte';

    const {children} = $props();

    // The shell (sidebar gutter + tab bar) belongs to the signed-in pages only:
    // the login screen is a standalone card, so it must not reserve the sidebar.
    const shell = $derived(app.ready && !!app.account && page.url.pathname !== '/welcome');

    onMount(() => {
        void app.init();
        const onUnauthorized = () => {
            app.account = null;
            app.showToast('登录已过期, 请重新登录', 'error');
            void goto('/welcome');
        };
        window.addEventListener('album:unauthorized', onUnauthorized);
        return () => window.removeEventListener('album:unauthorized', onUnauthorized);
    });

    $effect(() => {
        if (!app.ready) return;
        const path = page.url.pathname;
        if (!app.account && path !== '/welcome') {
            void goto('/welcome');
        } else if (app.account && path === '/welcome') {
            void goto('/photos');
        }
    });
</script>

{#if !app.ready}
    <div class="grid min-h-dvh place-items-center bg-light text-dark">
        <div class="flex flex-col items-center gap-4">
            <div class="grid size-16 animate-pulse place-items-center rounded-2xl bg-primary text-on-primary">
                <Cloud size={30} />
            </div>
            <p class="text-sm text-muted">正在载入 Source...</p>
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
