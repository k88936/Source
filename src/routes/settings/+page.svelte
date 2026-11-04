<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { api } from '#lib/api/client';
	import type { AppInfo } from '#lib/api/bindings/AppInfo';
	import type { CacheStats } from '#lib/api/bindings/CacheStats';
	import ConfirmDialog from '#lib/components/ConfirmDialog.svelte';
	import Icon from '#lib/components/Icon.svelte';
	import TopBar from '#lib/components/TopBar.svelte';
	import { formatBytes } from '#lib/format';
	import { app } from '#lib/state/app.svelte';

	let cache = $state<CacheStats | null>(null);
	let info = $state<AppInfo | null>(null);
	let theme = $state<'dark' | 'light'>('dark');
	let clearing = $state(false);
	let confirmAction = $state<'clearCache' | 'logout' | null>(null);

	const confirmCopy = $derived(
		confirmAction === 'logout'
			? {
					title: '退出登录',
					message: '本地索引会保留，下次登录后可以继续使用。',
					confirmLabel: '退出登录',
				}
			: {
					title: '清除缓存',
					message: '清除所有缩略图缓存？下次浏览会重新下载。',
					confirmLabel: '清除',
				},
	);

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
			app.showToast('缩略图缓存已清除', 'success');
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

<svelte:head><title>设置 · Source</title></svelte:head>

<TopBar title="设置" back />

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
				刷新状态
			</button>
			<button
				onclick={() => (confirmAction = 'logout')}
				class="flex items-center gap-1.5 rounded-xl px-3 py-2 text-xs font-medium text-danger transition-colors hover:bg-danger/10"
			>
				<Icon name="logout" size={15} /> 退出登录
			</button>
		</div>
	</section>

	<section class="mt-3 overflow-hidden rounded-2xl bg-gray">
		<div class="flex items-center justify-between px-4 py-3">
			<div>
				<p class="text-sm font-medium">外观</p>
				<p class="text-[11px] text-muted">跟随手动选择，默认深色</p>
			</div>
			<div class="flex rounded-lg bg-light p-0.5">
				<button
					onclick={() => setTheme('dark')}
					class="rounded-md px-3 py-1 text-xs transition-colors {theme === 'dark' ? 'bg-primary text-on-primary' : 'text-muted'}"
				>
					深色
				</button>
				<button
					onclick={() => setTheme('light')}
					class="rounded-md px-3 py-1 text-xs transition-colors {theme === 'light' ? 'bg-primary text-on-primary' : 'text-muted'}"
				>
					浅色
				</button>
			</div>
		</div>
	</section>

	<section class="mt-3 rounded-2xl bg-gray p-4">
		<div class="flex items-center justify-between">
			<div>
				<p class="text-sm font-medium">缩略图缓存</p>
				<p class="text-[11px] text-muted">仅缓存缩略图，原图从不写入本机</p>
			</div>
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
			<span>{cache?.thumbFiles ?? 0} 个文件</span>
			<span>上限 {formatBytes(cache?.budgetBytes ?? 512 * 1024 * 1024)}</span>
		</div>
		<button
			onclick={() => (confirmAction = 'clearCache')}
			disabled={clearing}
			class="mt-3 w-full rounded-xl border border-line py-2 text-xs font-medium text-danger transition-colors hover:bg-light disabled:opacity-50"
		>
			{clearing ? '清除中…' : '清除缓存'}
		</button>
	</section>

	<section class="mt-3 rounded-2xl bg-gray px-4 py-3 text-xs text-muted">
		<div class="flex justify-between py-1.5">
			<span>应用</span><span>{info?.name ?? 'Source'}</span>
		</div>
		<div class="flex justify-between py-1.5">
			<span>版本</span><span>{info?.version ?? '0.1.0'}</span>
		</div>
		<div class="flex justify-between py-1.5">
			<span>平台</span><span>{info?.platform ?? 'web'}</span>
		</div>
	</section>

	<p class="mt-4 text-center text-[11px] leading-relaxed text-muted">
		原图始终留在 Cloudreve · 相册就是云端文件夹
	</p>
</div>

<ConfirmDialog
	open={confirmAction !== null}
	title={confirmCopy.title}
	message={confirmCopy.message}
	confirmLabel={confirmCopy.confirmLabel}
	danger
	onclose={() => (confirmAction = null)}
	onconfirm={() => void runConfirm()}
/>
