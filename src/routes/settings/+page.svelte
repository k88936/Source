<script lang="ts">
    import {onMount} from 'svelte';
    import {goto} from '$app/navigation';
    import {api} from '#lib/api/client';
    import type {AppInfo} from '#lib/api/bindings/AppInfo';
    import type {CacheStats} from '#lib/api/bindings/CacheStats';
    import ConfirmDialog from '#lib/components/ConfirmDialog.svelte';
    import {LogOut} from '@lucide/svelte';
    import TopBar from '#lib/components/TopBar.svelte';
    import {formatBytes} from '#lib/format';
    import {app} from '#lib/state/app.svelte';
    import {i18n} from '#lib/state/i18n.svelte';
    import type {Locale} from '#lib/i18n/bundle';
    import {settings} from './page.i18n';

    type ConfirmAction = 'clearCache' | 'logout';

    let cache = $state<CacheStats | null>(null);
    let info = $state<AppInfo | null>(null);
    let theme = $state<'dark' | 'light'>('dark');
    let clearing = $state(false);
    let confirmAction = $state<ConfirmAction | null>(null);

    const t = $derived(i18n.pick(settings));

    const LOCALES: {value: Locale; label: string}[] = [
        {value: 'zh', label: '中文'},
        {value: 'en', label: 'English'},
    ];

    const confirmCopy = $derived(confirmAction === null ? null : t.confirm[confirmAction]);

    onMount(() => {
        void load();
        if (document.documentElement.classList.contains('light')) theme = 'light';
    });

    async function load() {
        try {
            [cache, info] = await Promise.all([api.cacheStats(), api.appInfo()]);
        } catch {
            // cache stats are optional
        }
    }

    async function clearCache() {
        clearing = true;
        try {
            await api.clearCache();
            await load();
            app.showToast(t.cacheCleared, 'success');
        } catch (err) {
            app.showToast(app.describeError(err), 'error');
        } finally {
            clearing = false;
        }
    }

    function setTheme(next: 'dark' | 'light') {
        theme = next;
        localStorage.setItem('immich-ui-theme', JSON.stringify(next));
        document.documentElement.classList.toggle('dark', next === 'dark');
        document.documentElement.classList.toggle('light', next === 'light');
    }

    async function logout() {
        await app.logout();
        await goto('/welcome');
    }

    async function runConfirm() {
        const action = confirmAction;
        confirmAction = null;
        if (action === 'clearCache') await clearCache();
        if (action === 'logout') await logout();
    }
</script>

<svelte:head><title>{t.title} - Source</title></svelte:head>

<TopBar title={t.title} back />

<div class="mx-auto max-w-3xl px-4 pb-28 pt-3">
    <section class="rounded-2xl bg-gray p-4">
        <div class="flex items-center gap-3">
            <div class="grid size-12 place-items-center rounded-full bg-primary text-lg font-semibold text-on-primary">
                {app.account?.user.name?.slice(0, 1).toUpperCase() ?? '?'}
            </div>
            <div class="min-w-0 flex-1">
                <p class="truncate text-sm font-semibold">{app.account?.user.name}</p>
                <p class="truncate text-xs text-muted">{app.account?.user.email}</p>
                <p class="truncate text-[11px] text-muted">{app.account?.server}</p>
            </div>
        </div>
        <div class="mt-3 flex gap-2">
            <button
                onclick={() => void app.refreshAccount()}
                class="flex-1 rounded-xl border border-line py-2 text-xs font-medium transition-colors hover:bg-line/40"
            >
                {t.refresh}
            </button>
            <button
                onclick={() => (confirmAction = 'logout')}
                class="flex items-center gap-1.5 rounded-xl px-3 py-2 text-xs font-medium text-danger transition-colors hover:bg-danger/10"
            >
                <LogOut size={15} />
                {t.logout}
            </button>
        </div>
    </section>

    <section class="mt-3 overflow-hidden rounded-2xl bg-gray">
        <div class="flex items-center justify-between px-4 py-3">
            <p class="text-sm font-medium">{t.appearance}</p>
            <div class="flex rounded-lg bg-light p-0.5">
                <button
                    onclick={() => setTheme('dark')}
                    class="rounded-md px-3 py-1 text-xs transition-colors {theme === 'dark'
                        ? 'bg-primary text-on-primary'
                        : 'text-muted'}"
                >
                    {t.dark}
                </button>
                <button
                    onclick={() => setTheme('light')}
                    class="rounded-md px-3 py-1 text-xs transition-colors {theme === 'light'
                        ? 'bg-primary text-on-primary'
                        : 'text-muted'}"
                >
                    {t.light}
                </button>
            </div>
        </div>
    </section>

    <section class="mt-3 overflow-hidden rounded-2xl bg-gray">
        <div class="flex items-center justify-between px-4 py-3">
            <p class="text-sm font-medium">{t.language}</p>
            <div class="flex rounded-lg bg-light p-0.5">
                {#each LOCALES as locale (locale.value)}
                    <button
                        onclick={() => i18n.set(locale.value)}
                        class="rounded-md px-3 py-1 text-xs transition-colors {i18n.locale === locale.value
                            ? 'bg-primary text-on-primary'
                            : 'text-muted'}"
                    >
                        {locale.label}
                    </button>
                {/each}
            </div>
        </div>
    </section>

    <section class="mt-3 overflow-hidden rounded-2xl bg-gray px-4 py-3">
        <div class="flex items-center justify-between gap-3">
            <p class="text-sm font-medium">{t.background}</p>
            <div class="flex shrink-0 rounded-lg bg-light p-0.5">
                <button
                    onclick={() => void app.setJobsPaused(false)}
                    class="rounded-md px-3 py-1 text-xs transition-colors {!app.jobsPaused
                        ? 'bg-primary text-on-primary'
                        : 'text-muted'}"
                >
                    {t.running}
                </button>
                <button
                    onclick={() => void app.setJobsPaused(true)}
                    class="rounded-md px-3 py-1 text-xs transition-colors {app.jobsPaused
                        ? 'bg-primary text-on-primary'
                        : 'text-muted'}"
                >
                    {t.paused}
                </button>
            </div>
        </div>
        <p class="mt-1 text-[11px] text-muted">{t.backgroundHint}</p>
    </section>

    <section class="mt-3 rounded-2xl bg-gray p-4">
        <div class="flex items-center justify-between">
            <p class="text-sm font-medium">{t.cache}</p>
            <span class="text-xs font-medium text-muted">{formatBytes(cache?.thumbBytes ?? 0)}</span>
        </div>
        <div class="mt-2 h-1.5 overflow-hidden rounded-full bg-muted/25">
            <div
                class="h-full rounded-full bg-primary"
                style:width="{cache && cache.budgetBytes > 0
                    ? Math.min(100, Math.round((cache.thumbBytes / cache.budgetBytes) * 100))
                    : 0}%"
            ></div>
        </div>
        <div class="mt-1 flex justify-between text-[11px] text-muted">
            <span>{t.files(cache?.thumbFiles ?? 0)}</span>
            <span>{t.limit(formatBytes(cache?.budgetBytes ?? 512 * 1024 * 1024))}</span>
        </div>
        <button
            onclick={() => (confirmAction = 'clearCache')}
            disabled={clearing}
            class="mt-3 w-full rounded-xl border border-line py-2 text-xs font-medium text-danger transition-colors hover:bg-light disabled:opacity-50"
        >
            {clearing ? t.clearing : t.clearCache}
        </button>
    </section>

    <section class="mt-3 rounded-2xl bg-gray px-4 py-3 text-xs text-muted">
        <div class="flex justify-between py-1.5">
            <span>{t.app}</span><span>{info?.name ?? 'Source'}</span>
        </div>
        <div class="flex justify-between py-1.5">
            <span>{t.version}</span><span>{info?.version ?? '0.1.0'}</span>
        </div>
        <div class="flex justify-between py-1.5">
            <span>{t.platform}</span><span>{info?.platform ?? 'web'}</span>
        </div>
    </section>
</div>

{#if confirmCopy}
    <ConfirmDialog
        open
        title={confirmCopy.title}
        message={confirmCopy.message}
        confirmLabel={confirmCopy.confirmLabel}
        danger
        onClose={() => (confirmAction = null)}
        onConfirm={() => void runConfirm()}
    />
{/if}
