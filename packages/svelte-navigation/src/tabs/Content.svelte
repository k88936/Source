<script lang="ts">
    import {setContext, type Component} from 'svelte';
    import {tabKey} from '../context';

    let {name, View}: {name: string; View: Component<Record<string, never>>} = $props();

    // svelte-ignore state_referenced_locally
    setContext(tabKey, name);
</script>

<View />
