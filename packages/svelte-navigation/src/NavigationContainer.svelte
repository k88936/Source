<script lang="ts">
    import {onMount, setContext, type Snippet} from 'svelte';
    import {rootKey} from './context';
    import {NavigationRoot} from './navigation.svelte';

    let {children}: {children: Snippet} = $props();

    const root = new NavigationRoot();
    setContext(rootKey, root);

    onMount(() => {
        root.start();
        return () => root.stop();
    });
</script>

{@render children()}
