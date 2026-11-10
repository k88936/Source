<script lang="ts">
    // Importing the plugin publishes `--safe-area-inset-*` on `:root`; it hooks the
    // keyboard events itself, so there is nothing to call.
    import '@saurl/tauri-plugin-safe-area-insets-css-api';
    import {Cloud} from '@lucide/svelte';
    import {onMount} from 'svelte';
    import {NavigationContainer} from 'svelte-navigation';
    import {listen} from '#lib/api/client';
    import Toast from '#lib/components/Toast.svelte';
    import {app} from '#lib/state/app.svelte';
    import {i18n} from '#lib/state/i18n.svelte';
    import MainTabs from './navigation/MainTabs.svelte';
    import WelcomeStack from './navigation/WelcomeStack.svelte';
    import {linking} from './navigation/linking';
    import {layout} from './navigation/layout.i18n';

    const t = $derived(i18n.pick(layout));

    onMount(() => {
        void app.init();
        // The adapter is the only place that can tell a refused session from a
        // wrong password, so it reports the loss and the tree reacts; nothing
        // here inspects an error kind.
        let unlisten: (() => void) | undefined;
        void listen('session://lost', () => {
            app.applySession({account: null, setup: 'offline'});
            app.showToast(t.sessionExpired, 'error');
        }).then((remove) => {
            unlisten = remove;
        });
        return () => unlisten?.();
    });
</script>

{#if !app.ready}
    <div class="grid h-full place-items-center bg-light text-dark">
        <div class="flex flex-col items-center gap-4">
            <div class="grid size-16 animate-pulse place-items-center rounded-2xl bg-primary text-on-primary">
                <Cloud size={30} />
            </div>
            <p class="text-sm text-muted">{t.loading}</p>
        </div>
    </div>
{:else}
    <!-- Signed in or not is what picks the root stack: the login screen has no tabs. -->
    <NavigationContainer {linking}>
        {#if app.account}
            <MainTabs />
        {:else}
            <WelcomeStack />
        {/if}
    </NavigationContainer>
{/if}

<Toast />
