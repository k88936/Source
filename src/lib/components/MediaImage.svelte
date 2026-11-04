<script lang="ts">
	import type { HTMLImgAttributes } from 'svelte/elements';

	import type { MediaSourceParams } from '#lib/api/bindings/MediaSourceParams';
	import { mediaSource } from '#lib/api/media';

	let {
		source,
		...rest
	}: Omit<HTMLImgAttributes, 'src'> & { source: MediaSourceParams } = $props();

	let src = $state('');

	$effect(() => {
		let live = true;
		void mediaSource(source).then((url) => {
			if (live) src = url;
		});
		return () => {
			live = false;
		};
	});
</script>

<img {...rest} src={src || undefined} />
