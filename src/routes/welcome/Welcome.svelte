<script lang="ts">
    import {api} from '#lib/api/client';
    import type {AccountState} from '#lib/api/bindings/AccountState';
    import {Cloud} from '@lucide/svelte';
    import {app} from '#lib/app/app.svelte';
    import {i18n} from '#lib/i18n/i18n.svelte';
    import {common} from '#lib/i18n/common.i18n';
    import {welcome} from './Welcome.i18n';

    let server = $state((typeof localStorage !== 'undefined' && localStorage.getItem('source-server')) || '');
    let email = $state('');
    let password = $state('');
    let otp = $state('');
    let sessionId = $state('');
    let step = $state<'credentials' | '2fa'>('credentials');
    let loading = $state(false);
    let error = $state('');

    const t = $derived(i18n.pick(welcome));
    const c = $derived(i18n.pick(common));

    async function connect() {
        error = '';
        if (!server.trim()) {
            error = t.errors.server;
            return;
        }
        if (!email.trim() || !password) {
            error = t.errors.credentials;
            return;
        }
        loading = true;
        try {
            const baseUrl = server.trim().replace(/\/+$/, '');
            await api.setServer({baseUrl});
            if (typeof localStorage !== 'undefined') localStorage.setItem('source-server', baseUrl);
            const outcome = await api.login({email: email.trim(), password});
            if (outcome.kind === 'twoFactorRequired') {
                sessionId = outcome.sessionId;
                step = '2fa';
                otp = '';
            } else {
                await finish(outcome.account);
            }
        } catch (err) {
            error = app.describeError(err);
        } finally {
            loading = false;
        }
    }

    async function verify() {
        error = '';
        if (otp.trim().length !== 6) {
            error = t.errors.otp;
            return;
        }
        loading = true;
        try {
            const account = await api.login2fa({sessionId, otp: otp.trim()});
            await finish(account);
        } catch (err) {
            error = app.describeError(err);
        } finally {
            loading = false;
        }
    }

    async function finish(account: AccountState) {
        app.applySession({account, setup: 'live'});
        await app.afterLogin();
    }
</script>

<svelte:head><title>{t.title} - Source</title></svelte:head>

<div class="flex min-h-dvh flex-col justify-center bg-light px-6 py-12 text-dark">
    <div class="mx-auto w-full max-w-sm">
        <div class="flex flex-col items-center text-center">
            <span class="grid size-14 place-items-center rounded-2xl bg-primary text-on-primary">
                <Cloud size={28} />
            </span>
            <h1 class="mt-4 text-xl font-semibold">Source</h1>
        </div>

        {#if step === 'credentials'}
            <form
                class="mt-10 flex flex-col gap-4"
                onsubmit={(event) => {
                    event.preventDefault();
                    void connect();
                }}
            >
                <label class="block">
                    <span class="mb-1.5 block text-xs font-medium text-muted">{t.server}</span>
                    <input
                        bind:value={server}
                        type="url"
                        autocomplete="url"
                        placeholder="https://cloud.example.com"
                        class="field"
                    />
                </label>
                <label class="block">
                    <span class="mb-1.5 block text-xs font-medium text-muted">{t.email}</span>
                    <input
                        bind:value={email}
                        type="email"
                        autocomplete="username"
                        placeholder="you@example.com"
                        class="field"
                    />
                </label>
                <label class="block">
                    <span class="mb-1.5 block text-xs font-medium text-muted">{t.password}</span>
                    <input
                        bind:value={password}
                        type="password"
                        autocomplete="current-password"
                        placeholder="********"
                        class="field"
                    />
                </label>

                {#if error}
                    <p class="rounded-lg bg-danger/10 px-3 py-2 text-xs text-danger">{error}</p>
                {/if}

                <button
                    type="submit"
                    disabled={loading}
                    class="mt-1 flex h-11 items-center justify-center rounded-xl bg-primary text-sm font-medium text-on-primary transition-opacity hover:opacity-90 disabled:opacity-50"
                >
                    {loading ? t.connecting : t.signIn}
                </button>
            </form>
        {:else}
            <form
                class="mt-10 flex flex-col gap-4"
                onsubmit={(event) => {
                    event.preventDefault();
                    void verify();
                }}
            >
                <div class="text-center">
                    <p class="text-sm font-medium">{t.twoFactor}</p>
                    <p class="mt-1 text-xs text-muted">{t.twoFactorHint}</p>
                </div>
                <input
                    bind:value={otp}
                    inputmode="numeric"
                    maxlength="6"
                    autocomplete="one-time-code"
                    placeholder="000000"
                    class="field py-3 text-center text-2xl tracking-[0.5em]"
                />
                {#if error}
                    <p class="rounded-lg bg-danger/10 px-3 py-2 text-xs text-danger">{error}</p>
                {/if}
                <button
                    type="submit"
                    disabled={loading}
                    class="flex h-11 items-center justify-center rounded-xl bg-primary text-sm font-medium text-on-primary transition-opacity hover:opacity-90 disabled:opacity-50"
                >
                    {loading ? t.verifying : t.verify}
                </button>
                <button
                    type="button"
                    onclick={() => {
                        step = 'credentials';
                        error = '';
                    }}
                    class="text-xs text-muted hover:text-dark"
                >
                    {c.back}
                </button>
            </form>
        {/if}
    </div>
</div>
