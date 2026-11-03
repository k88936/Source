<script lang="ts">
	import type { LocalAlbum } from '#lib/api/bindings/LocalAlbum';
	import type { MediaPermissionState } from '#lib/api/bindings/MediaPermissionState';
	import Icon from './Icon.svelte';

	let {
		albums,
		permission,
		onrequest,
		onupload,
	}: {
		albums: LocalAlbum[];
		permission: MediaPermissionState | null;
		onrequest: () => void;
		onupload: () => void;
	} = $props();
</script>

<section class="mt-5">
	<div class="mb-2 flex items-center justify-between px-1">
		<h2 class="text-sm font-medium">本机相册</h2>
		<button onclick={onupload} class="text-xs font-medium text-primary">
			+ 上传
		</button>
	</div>

	{#if permission?.granted === 'denied'}
		<div class="rounded-2xl bg-gray p-4 text-center">
			<p class="text-sm font-medium">需要访问本机照片</p>
			<p class="mt-1 text-xs text-muted">授权后才能浏览和上传本机相册。</p>
			<button
				onclick={onrequest}
				class="mt-3 rounded-xl bg-primary px-4 py-2 text-xs font-medium text-on-primary"
			>
				授权访问
			</button>
		</div>
	{:else if albums.length === 0}
		<div class="rounded-2xl bg-gray p-5 text-center text-xs text-muted">没有找到本机相册</div>
	{:else}
		<div class="grid grid-cols-2 gap-3 sm:grid-cols-3">
			{#each albums as album (album.id)}
				<a
					href="/library/{encodeURIComponent(album.id)}"
					class="block rounded-2xl bg-gray p-4 transition-colors hover:bg-line/40"
				>
					<Icon name="folder" size={22} class="text-muted" />
					<p class="mt-6 truncate text-sm font-medium">{album.name}</p>
					<p class="text-xs text-muted">{album.assetCount} 项</p>
				</a>
			{/each}
		</div>
	{/if}
</section>
