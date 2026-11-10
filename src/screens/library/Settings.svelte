<script lang="ts">
    import {Avatar, Button, FormatBytes, Meter, Select, Switch, ThemePreference, themeManager} from '@immich/ui';
    import {mdiLogout, mdiRefresh} from '@mdi/js';
    import {onMount} from 'svelte';
    import {api} from '#lib/api/client';
    import type {AppInfo} from '#lib/api/bindings/AppInfo';
    import type {CacheStats} from '#lib/api/bindings/CacheStats';
    import ConfirmDialog from '#lib/components/ConfirmDialog.svelte';
    import type {Locale} from '#lib/i18n/bundle';
    import {app} from '#lib/state/app.svelte';
    import {i18n} from '#lib/state/i18n.svelte';
    import ScreenOptions from '#navigation/ScreenOptions.svelte';
    import {settings} from './Settings.i18n';

    type ConfirmAction = 'clearCache' | 'logout';

    let cache = $state<CacheStats | null>(null);
    let info = $state<AppInfo | null>(null);
    let clearing = $state(false);
    let confirmAction = $state<ConfirmAction | null>(null);

    const t = $derived(i18n.pick(settings));

    const LOCALES: {value: Locale; label: string}[] = [
        {value: 'zh', label: '中文'},
        {value: 'en', label: 'English'},
    ];

    const THEMES = $derived([
        {value: ThemePreference.System, label: t.system},
        {value: ThemePreference.Light, label: t.light},
        {value: ThemePreference.Dark, label: t.dark},
    ]);

    const confirmCopy = $derived(confirmAction === null ? null : t.confirm[confirmAction]);

    onMount(() => {
        void load();
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

    async function logout() {
        await app.logout();
    }

    async function runConfirm() {
        const action = confirmAction;
        confirmAction = null;
        if (action === 'clearCache') await clearCache();
        if (action === 'logout') await logout();
    }
</script>

<svelte:head><title>{t.title} - Source</title></svelte:head>

<ScreenOptions title={t.title} />

<div class="mx-auto max-w-3xl px-4 pb-28 pt-3">
    <section class="rounded-2xl bg-gray p-4">
        <div class="flex items-center gap-3">
            <Avatar name={app.account?.user.name ?? ''} size="large" />
            <div class="min-w-0 flex-1">
                <p class="truncate text-sm font-semibold">{app.account?.user.name}</p>
                <p class="truncate text-xs text-muted">{app.account?.user.email}</p>
                <p class="truncate text-[11px] text-muted">{app.account?.server}</p>
            </div>
        </div>
        <div class="mt-3 flex gap-2">
            <Button
                size="small"
                variant="ghost"
                fullWidth
                leadingIcon={mdiRefresh}
                onclick={() => void app.refreshAccount()}
            >
                {t.refresh}
            </Button>
            <Button
                size="small"
                variant="ghost"
                color="danger"
                fullWidth
                leadingIcon={mdiLogout}
                onclick={() => (confirmAction = 'logout')}
            >
                {t.logout}
            </Button>
        </div>
    </section>

    <section class="mt-3 rounded-2xl bg-gray px-4 py-3">
        <div class="flex items-center justify-between gap-3">
            <p class="shrink-0 text-sm font-medium">{t.appearance}</p>
            <Select
                class="w-28"
                size="small"
                options={THEMES}
                value={themeManager.preference}
                onChange={(preference) => themeManager.setPreference(preference)}
            />
        </div>
    </section>

    <section class="mt-3 rounded-2xl bg-gray px-4 py-3">
        <div class="flex items-center justify-between gap-3">
            <p class="shrink-0 text-sm font-medium">{t.language}</p>
            <Select
                class="w-28"
                size="small"
                options={LOCALES}
                value={i18n.locale}
                onChange={(locale) => i18n.set(locale)}
            />
        </div>
    </section>

    <section class="mt-3 rounded-2xl bg-gray px-4 py-3">
        <div class="flex items-center justify-between gap-3">
            <p class="shrink-0 text-sm font-medium">{t.background}</p>
            <Switch
                checked={!app.jobsPaused}
                aria-label={t.background}
                onCheckedChange={(running) => void app.setJobsPaused(!running)}
            />
        </div>
    </section>

    <section class="mt-3 rounded-2xl bg-gray p-4">
        <div class="flex items-center justify-between">
            <p class="text-sm font-medium">{t.cache}</p>
            <p class="text-xs font-medium text-muted"><FormatBytes bytes={cache?.thumbBytes ?? 0} /></p>
        </div>
        <Meter
            class="mt-2"
            size="small"
            value={cache?.thumbBytes ?? 0}
            max={Math.max(1, cache?.budgetBytes ?? 1)}
        />
        <div class="mt-1 flex justify-between text-[11px] text-muted">
            <span>{t.files(cache?.thumbFiles ?? 0)}</span>
            <span>{t.limitPrefix} <FormatBytes bytes={cache?.budgetBytes ?? 512 * 1024 * 1024} /></span>
        </div>
        <Button
            class="mt-3"
            size="small"
            variant="ghost"
            color="danger"
            fullWidth
            loading={clearing}
            onclick={() => (confirmAction = 'clearCache')}
        >
            {clearing ? t.clearing : t.clearCache}
        </Button>
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
