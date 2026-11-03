<script lang="ts">
	import { api } from '#lib/api/client';
	import type { RemoteFolderPage } from '#lib/api/bindings/RemoteFolderPage';
	import Icon from './Icon.svelte';
	import { app } from '#lib/state/app.svelte';

	let { open = $bindable(false), onchange }: { open?: boolean; onchange?: () => void } = $props();

	let browse = $state<RemoteFolderPage | null>(null);
	let loading = $state(false);
	let error = $state('');
	let saving = $state(false);

	$effect(() => {
		if (open) void browseTo('cloudreve://my');
	});

	async function browseTo(uri: string) {
		loading = true;
		error = '';
		browse = null;
		try {
			browse = await api.listRemoteFolders({ uri, pageToken: null });
		} catch (err) {
			error = app.describeError(err);
		} finally {
			loading = false;
		}
	}

	async function choose() {
		if (!browse?.parent || saving) return;
		saving = true;
		try {
			const root = await api.setLibraryRoot({ uri: browse.parent.uri });
			if (app.session) app.session = { ...app.session, libraryRoot: root };
			open = false;
			app.showToast(`图库根目录已设为 ${root.path}`, 'success');
			onchange?.();
		} catch (err) {
			error = app.describeError(err);
		} finally {
			saving = false;
		}
	}
</script>

{#if open}
	<div
		class="fixed inset-0 z-50 flex items-end justify-center bg-black/50 sm:items-center sm:p-4"
		role="dialog"
		aria-modal="true"
	>
		<div
			class="flex h-[80dvh] w-full max-w-lg flex-col rounded-t-3xl bg-light sm:h-[70dvh] sm:rounded-3xl"
		>
			<div class="flex items-center justify-between px-4 pb-2 pt-3">
				<div>
 				<h2 class="text-base font-semibold">选择图库根目录</h2>
					<p class="text-xs text-muted">
						当前：{browse?.parent?.path ?? '/'}
					</p>
				</div>
				<button
					onclick={() => (open = false)}
					class="grid size-9 place-items-center rounded-full bg-gray text-dark hover:bg-line/40"
					aria-label="关闭"
				>
					<Icon name="close" size={18} />
				</button>
			</div>

			<div class="flex-1 overflow-y-auto px-2">
				{#if browse?.parent}
					<button
						onclick={() => void browseTo(browse?.parent?.uri ?? 'cloudreve://my')}
						class="mb-1 flex w-full items-center gap-3 rounded-xl px-3 py-2.5 text-left text-sm hover:bg-gray"
					>
						<Icon name="back" size={18} class="text-muted" /> 上一级
					</button>
				{/if}
				{#if loading}
					<div class="flex justify-center py-12">
						<div
							class="size-6 animate-spin rounded-full border-2 border-line border-t-primary"
						></div>
					</div>
				{:else if error}
					<p class="px-3 py-6 text-center text-xs text-red-500">{error}</p>
				{:else if browse && browse.folders.length === 0}
					<p class="px-3 py-8 text-center text-xs text-muted">这个目录下没有子文件夹</p>
				{:else}
					{#each browse?.folders ?? [] as folder (folder.id)}
						<button
							onclick={() => void browseTo(folder.uri)}
							class="flex w-full items-center gap-3 rounded-xl px-3 py-2.5 text-left text-sm hover:bg-gray"
						>
							<Icon name="folder" size={20} class="text-amber-500" />
							<span class="min-w-0 flex-1 truncate">{folder.name}</span>
							<Icon name="forward" size={15} class="text-muted" />
						</button>
					{/each}
				{/if}
			</div>

			<div
				class="border-t border-line p-4 pb-[calc(1rem+var(--safe-bottom))]"
			>
				<button
					onclick={() => void choose()}
					disabled={!browse?.parent || saving}
					class="h-11 w-full rounded-xl bg-primary text-sm font-semibold text-on-primary transition-opacity hover:opacity-90 disabled:opacity-40"
				>
					{saving
						? '正在设置…'
						: browse?.parent
							? `使用 ${browse.parent.name} 作为图库根`
							: '请先进入一个文件夹'}
				</button>
			</div>
		</div>
	</div>
{/if}
