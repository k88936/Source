<script lang="ts">
    import type {HTMLImgAttributes} from 'svelte/elements';

    import type {MediaSourceParams} from '#lib/api/bindings/MediaSourceParams';
    import {mediaSource} from '#lib/api/media';
    import {app} from '#lib/app/app.svelte';

    let {source, ...rest}: Omit<HTMLImgAttributes, 'src'> & {source: MediaSourceParams} = $props();

    let src = $state('');

    // The address is minted for the account the page shows, but only a live
    // client can honour it: an address asked for during the start-up check is
    // refused, and minting it again when the answer arrives is the retry.
    $effect(() => {
        void app.account?.user.id;
        void app.setup;
        let live = true;
        void mediaSource(source).then((url) => {
            if (live) src = url;
        });
        return () => {
            live = false;
        };
    });
</script>

<!-- The address does not change with the session, so the element that asked
before the check finished would keep its refusal forever. -->
{#key app.setup}
    <img {...rest} src={src || undefined} />
{/key}
