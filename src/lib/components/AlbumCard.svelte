<script lang="ts">
	import type { AlbumSummary } from '#lib/api/bindings/AlbumSummary';
	import Icon from './Icon.svelte';
	import MediaImage from './MediaImage.svelte';

	let { album }: { album: AlbumSummary } = $props();

	let covers = $derived(album.coverAssetIds.slice(0, 4));
</script>

<a
	href="/albums/{album.id}"
	class="group block rounded-2xl border border-transparent p-4 transition-colors hover:border-line hover:bg-gray sidebar:p-5"
>
	<div class="relative aspect-square overflow-hidden rounded-xl bg-gray">
		{#if covers.length > 0}
			<!-- 2×2 mosaic, like the reference app: one image reads as a single
			     photo, four read as a collection. -->
			<div class="grid size-full grid-cols-2 grid-rows-2 gap-[2px]">
				{#each covers as cover, index (cover)}
					<MediaImage
						source={{ kind: 'thumb', assetId: cover }}
						alt=""
						loading="lazy"
						decoding="async"
						class="size-full object-cover {covers.length === 1
							? 'col-span-2 row-span-2'
							: covers.length === 3 && index === 0
								? 'row-span-2'
								: covers.length === 2
									? 'row-span-2'
									: ''}"
					/>
				{/each}
			</div>
		{:else}
			<div class="grid size-full place-items-center text-muted">
				<Icon name="albums" size={40} />
			</div>
		{/if}
	</div>
	<div class="pt-4">
		<p class="line-clamp-2 text-base font-semibold transition-colors group-hover:text-primary">
			{album.name}
		</p>
		<p class="mt-1 truncate text-xs text-muted">{album.assetCount} 项 · {album.path}</p>
	</div>
</a>
