<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { api } from '#lib/api/client';
	import type { MediaPermissionState } from '#lib/api/bindings/MediaPermissionState';
	import type { LocalAlbum } from '#lib/api/bindings/LocalAlbum';
	import CapacityCard from '#lib/components/CapacityCard.svelte';
	import Icon from '#lib/components/Icon.svelte';
	import IndexButton from '#lib/components/IndexButton.svelte';
	import IndexCard from '#lib/components/IndexCard.svelte';
	import LocalAlbumGrid from '#lib/components/LocalAlbumGrid.svelte';
	import RootPicker from '#lib/components/RootPicker.svelte';
	import TopBar from '#lib/components/TopBar.svelte';
	import UploadQueueCard from '#lib/components/UploadQueueCard.svelte';
	import UploadSheet from '#lib/components/UploadSheet.svelte';
	import { app } from '#lib/state/app.svelte';

	let rootOpen = $state(false);
	let localAlbums = $state<LocalAlbum[]>([]);
	let permission = $state<MediaPermissionState | null>(null);
	let uploadOpen = $state(false);
	let initialDest = $state('');

	onMount(() => {
		if (page.url.searchParams.get('setup') === '1' || !app.session?.libraryRoot) {
			rootOpen = true;
		}
		if (page.url.searchParams.get('upload') === '1') {
			uploadOpen = true;
		}
		void loadLocal();
		void refreshCapacity();
	});

	async function refreshCapacity() {
		if (!app.session) return;
		try {
			const capacity = await api.refreshCapacity();
			app.session = { ...app.session, capacity };
		} catch {
			// capacity is informational
		}
	}

	async function loadLocal() {
		try {
			permission = await api.mediaPermissionState();
			if (permission.granted !== 'denied') {
				localAlbums = await api.localAlbumList();
			}
		} catch {
			localAlbums = [];
		}
	}

	async function requestPermission() {
		try {
			permission = await api.requestMediaPermission();
			if (permission.granted !== 'denied') await loadLocal();
		} catch (err) {
			app.showToast(app.describeError(err), 'error');
		}
	}
</script>

<svelte:head><title>资料库 · Source</title></svelte:head>

<TopBar title="资料库" subtitle={app.session?.user.email ?? ''}>
	{#snippet actions()}
		<IndexButton />
		<button
			onclick={() => void goto('/settings')}
			class="grid size-9 place-items-center rounded-full text-dark transition-colors hover:bg-gray"
			aria-label="设置"
		>
			<Icon name="settings" size={21} />
		</button>
	{/snippet}
</TopBar>

<div class="mx-auto max-w-3xl px-4 pb-28 pt-3">
	<CapacityCard />

	<section class="mt-3 rounded-2xl bg-gray p-4">
		<div class="flex items-center gap-3">
			<div class="grid size-10 shrink-0 place-items-center rounded-xl bg-primary/10 text-primary">
				<Icon name="folder" size={21} />
			</div>
			<div class="min-w-0 flex-1">
				<p class="text-sm font-semibold">图库根目录</p>
				<p class="truncate text-xs text-muted">
					{app.session?.libraryRoot?.path ?? '尚未选择'}
				</p>
			</div>
			<button
				onclick={() => (rootOpen = true)}
				class="rounded-lg border border-line px-3 py-1.5 text-xs font-medium text-dark transition-colors hover:bg-line/40"
			>
				{app.session?.libraryRoot ? '更改' : '选择'}
			</button>
		</div>
		<p class="mt-2 text-[11px] text-muted">
			索引该目录下所有图片与视频；其子目录会成为相册。
		</p>
	</section>

	<IndexCard />

	<UploadQueueCard />

	<LocalAlbumGrid
		albums={localAlbums}
		{permission}
		onrequest={() => void requestPermission()}
		onupload={() => (uploadOpen = true)}
	/>

	<section class="mt-5 rounded-2xl bg-gray p-1">
		<a href="/settings" class="flex items-center gap-3 rounded-xl px-3 py-3 text-sm hover:bg-line/40">
			<Icon name="settings" size={20} class="text-muted" />
			<span class="flex-1">设置与缓存</span>
			<Icon name="forward" size={16} class="text-muted" />
		</a>
	</section>
</div>

<RootPicker bind:open={rootOpen} onchange={() => void app.startIndex(false)} />

<UploadSheet bind:open={uploadOpen} bind:initialDest={initialDest} />
