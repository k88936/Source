<script lang="ts">
    import {goto} from '$app/navigation';
    import {api} from '#lib/api/client';
    import type {AccountState} from '#lib/api/bindings/AccountState';
    import Icon from '#lib/components/Icon.svelte';
    import {app} from '#lib/state/app.svelte';

    let server = $state((typeof localStorage !== 'undefined' && localStorage.getItem('album-server')) || '');
    let email = $state('');
    let password = $state('');
    let otp = $state('');
    let sessionId = $state('');
    let step = $state<'credentials' | '2fa'>('credentials');
    let loading = $state(false);
    let error = $state('');

    async function connect() {
        error = '';
        if (!server.trim()) {
            error = '请输入 Cloudreve 服务器地址';
            return;
        }
        if (!email.trim() || !password) {
            error = '请输入邮箱和密码';
            return;
        }
        loading = true;
        try {
            const baseUrl = server.trim().replace(/\/+$/, '');
            await api.setServer({baseUrl});
            if (typeof localStorage !== 'undefined') localStorage.setItem('album-server', baseUrl);
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
            error = '请输入 6 位验证码';
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
        app.account = account;
        await app.afterLogin();
        await goto('/photos');
    }
</script>

<svelte:head><title>登录 - Source</title></svelte:head>

<div class="flex min-h-dvh flex-col justify-center bg-light px-6 py-12 text-dark">
    <div class="mx-auto w-full max-w-sm">
        <div class="flex flex-col items-center text-center">
            <span class="grid size-14 place-items-center rounded-2xl bg-primary text-on-primary">
                <Icon name="cloud" size={28} />
            </span>
            <h1 class="mt-4 text-xl font-semibold">Source</h1>
            <p class="mt-1 text-sm text-muted">用 Cloudreve 账号登录, 浏览你的相册</p>
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
                    <span class="mb-1.5 block text-xs font-medium text-muted">服务器地址</span>
                    <input
                        bind:value={server}
                        type="url"
                        autocomplete="url"
                        placeholder="https://cloud.example.com"
                        class="field"
                    />
                </label>
                <label class="block">
                    <span class="mb-1.5 block text-xs font-medium text-muted">邮箱</span>
                    <input
                        bind:value={email}
                        type="email"
                        autocomplete="username"
                        placeholder="you@example.com"
                        class="field"
                    />
                </label>
                <label class="block">
                    <span class="mb-1.5 block text-xs font-medium text-muted">密码</span>
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
                    {loading ? '正在连接...' : '登录'}
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
                    <p class="text-sm font-medium">两步验证</p>
                    <p class="mt-1 text-xs text-muted">输入身份验证器中的 6 位验证码</p>
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
                    {loading ? '验证中...' : '验证并登录'}
                </button>
                <button
                    type="button"
                    onclick={() => {
                        step = 'credentials';
                        error = '';
                    }}
                    class="text-xs text-muted hover:text-dark"
                >
                    返回
                </button>
            </form>
        {/if}

        <p class="mt-10 text-center text-[11px] leading-relaxed text-muted">
            连接你的 Cloudreve 私有云 - 原图不会缓存在本机
        </p>
    </div>
</div>
