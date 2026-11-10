<script lang="ts" generics="ParamList extends ParamListBase">
    import {getContext, onDestroy, type Component} from 'svelte';
    import {stackKey} from '../context';
    import type {StackController, StackScreenConfig} from '../navigation.svelte';
    import type {ParamListBase, Route, StackOptions} from '../types';

    let {
        name,
        component,
        options,
    }: {
        name: keyof ParamList & string;
        component: Component<Record<string, never>>;
        options?: StackOptions | ((route: Route<ParamList>) => StackOptions);
    } = $props();

    const controller = getContext<StackController>(stackKey);

    // Registration runs during the navigator's render, so its first layer already knows the screen.
    // svelte-ignore state_referenced_locally
    const unregister = controller.register({name, component, options} as StackScreenConfig);

    onDestroy(unregister);
</script>
