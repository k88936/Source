<script lang="ts">
    import DemoFrame from './lib/DemoFrame.svelte';
    import {i18n} from './lib/i18n.svelte';
    import {text} from './lib/text';
    import {theme} from './lib/theme.svelte';

    const t = $derived(i18n.pick(text));

    /** The app reads the locale once at boot, so only the site's own switch reloads the frame. */
    let reload = $state(0);

    function switchLocale(): void {
        i18n.set(i18n.other);
        reload += 1;
    }
</script>

<svelte:head>
    <title>Source</title>
</svelte:head>

<div class="page">
    <header class="top">
        <span class="brand">Source</span>
        <div class="actions">
            <div class="seg">
                <button class:active={theme.value === 'dark'} onclick={() => theme.set('dark')}>
                    {t.dark}
                </button>
                <button class:active={theme.value === 'light'} onclick={() => theme.set('light')}>
                    {t.light}
                </button>
            </div>
            <button class="lang" onclick={switchLocale}>{t.switchLabel}</button>
        </div>
    </header>

    <main class="preview">
        <DemoFrame {reload} />
    </main>

    <footer class="bottom">
        <a class="credit" href="https://www.bing.com" target="_blank" rel="noreferrer">{t.credit}</a>
    </footer>
</div>
