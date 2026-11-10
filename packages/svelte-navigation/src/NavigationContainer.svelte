<script lang="ts">
    import {onMount, setContext, type Snippet} from 'svelte';
    import {rootKey} from './context';
    import {NavigationRoot, type Linking} from './navigation.svelte';

    let {linking = {}, children}: {linking?: Linking; children: Snippet} = $props();

    // svelte-ignore state_referenced_locally
    const root = new NavigationRoot(linking);
    setContext(rootKey, root);

    onMount(() => {
        root.start();
        return () => root.stop();
    });
</script>

{@render children()}
