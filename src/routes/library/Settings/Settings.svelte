<script lang="ts">
    import {Avatar, Button, FormatBytes, IconButton, Meter, Select, Switch, ThemePreference, themeManager} from '@immich/ui';
    import {mdiLogout, mdiRefresh, mdiSync} from '@mdi/js';
    import {onMount} from 'svelte';
    import {api} from '#lib/api/client';
    import type {AppInfo} from '#lib/api/bindings/AppInfo';
    import type {CacheStats} from '#lib/api/bindings/CacheStats';
    import type {Frequency} from '#lib/api/bindings/Frequency';
    import type {Settings} from '#lib/api/bindings/Settings';
    import type {SettingsDelta} from '#lib/api/bindings/SettingsDelta';
    import ConfirmDialog from '#lib/components/overlay/ConfirmDialog/ConfirmDialog.svelte';
    import type {Locale} from '#lib/i18n/bundle';
    import {app} from '#lib/app/app.svelte';
    import {i18n} from '#lib/i18n/i18n.svelte';
    import ScreenOptions from '#lib/components/shell/ScreenOptions/ScreenOptions.svelte';
    import {common} from '#lib/i18n/common.i18n';
    import {settings} from './Settings.i18n';

    type ConfirmAction = 'clearCache' | 'logout';

    let cache = $state<CacheStats | null>(null);
    let info = $state<AppInfo | null>(null);
    /// What Rust owns: the background switch and the two frequencies. It is the
    /// authority, so every change is written and then read back.
    let prefs = $state<Settings | null>(null);
    let clearing = $state(false);
    let running = $state<'local' | 'cloud' | null>(null);
    let confirmAction = $state<ConfirmAction | null>(null);

    const t = $derived(i18n.pick(settings));
    const c = $derived(i18n.pick(common));

    const LOCALES: {value: Locale; label: string}[] = [
        {value: 'zh', label: '中文'},
        {value: 'en', label: 'English'},
    ];

    const THEMES = $derived([
        {value: ThemePreference.System, label: t.system},
        {value: ThemePreference.Light, label: t.light},
        {value: ThemePreference.Dark, label: t.dark},
    ]);

    /// Never is the default: a schedule is the user saying the app may work on
    /// its own, and that is not a decision to make for them.
    const FREQUENCIES = $derived<{value: Frequency; label: string}[]>([
        {value: 'never', label: t.never},
        {value: 'daily', label: t.daily},
        {value: 'weekly', label: t.weekly},
        {value: 'monthly', label: t.monthly},
    ]);

    const confirmCopy = $derived(confirmAction === null ? null : t.confirm[confirmAction]);

    onMount(() => {
        void load();
    });

    async function load() {
        try {
            [cache, info, prefs] = await Promise.all([api.cacheStats(), api.appInfo(), api.settings()]);
        } catch (err) {
            app.showToast(app.describeError(err), 'error');
        }
    }

    /// Write a delta, then read the settings back: the answer is the row Rust
    /// holds, not a guess about what the write meant.
    async function update(delta: SettingsDelta) {
        try {
            await api.setSettings(delta);
            prefs = await api.settings();
        } catch (err) {
            app.showToast(app.describeError(err), 'error');
        }
    }

    /// Run one pass now. It does not move the due date: only the schedule does,
    /// so a manual run never delays the next automatic one.
    async function runNow(which: 'local' | 'cloud') {
        running = which;
        try {
            const count = which === 'local' ? await api.syncStart() : await api.indexStart();
            app.showToast(t.queued(count), 'success');
        } catch (err) {
            app.showToast(app.describeError(err), 'error');
        } finally {
            running = null;
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

<svelte:head><title>{c.settings} - Source</title></svelte:head>

<ScreenOptions title={c.settings} />

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
                checked={prefs?.keepAlive ?? true}
                aria-label={t.background}
                onCheckedChange={(hold) => void update({keepAlive: hold})}
            />
        </div>
    </section>

    <section class="mt-3 rounded-2xl bg-gray px-4 py-3">
        <p class="text-sm font-medium">{t.schedule}</p>
        <div class="mt-2 flex items-center justify-between gap-3">
            <p class="shrink-0 text-[13px] text-muted">{t.localScan}</p>
            <div class="flex items-center gap-1">
                <IconButton
                    icon={mdiSync}
                    aria-label={t.runNow}
                    size="small"
                    variant="ghost"
                    loading={running === 'local'}
                    onclick={() => void runNow('local')}
                />
                <Select
                    class="w-24"
                    size="small"
                    options={FREQUENCIES}
                    value={prefs?.syncFrequency ?? 'never'}
                    onChange={(frequency) => void update({syncFrequency: frequency})}
                />
            </div>
        </div>
        <div class="mt-2 flex items-center justify-between gap-3">
            <p class="shrink-0 text-[13px] text-muted">{t.cloudIndex}</p>
            <div class="flex items-center gap-1">
                <IconButton
                    icon={mdiSync}
                    aria-label={t.runNow}
                    size="small"
                    variant="ghost"
                    loading={running === 'cloud'}
                    onclick={() => void runNow('cloud')}
                />
                <Select
                    class="w-24"
                    size="small"
                    options={FREQUENCIES}
                    value={prefs?.indexFrequency ?? 'never'}
                    onChange={(frequency) => void update({indexFrequency: frequency})}
                />
            </div>
        </div>
    </section>

    <section class="mt-3 rounded-2xl bg-gray p-4">
        <div class="flex items-center justify-between">
            <p class="text-sm font-medium">{t.cache}</p>
            <p class="text-xs font-medium text-muted"><FormatBytes bytes={cache?.thumbBytes ?? 0} /></p>
        </div>
        <Meter class="mt-2" size="small" value={cache?.thumbBytes ?? 0} max={Math.max(1, cache?.budgetBytes ?? 1)} />
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
