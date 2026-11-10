<script lang="ts">
    import {createBottomTabNavigator, type TabBarProps} from 'svelte-navigation';
    import AlbumsStack from './AlbumsStack.svelte';
    import LibraryStack from './LibraryStack.svelte';
    import PhotosStack from './PhotosStack.svelte';
    import SyncStack from './SyncStack.svelte';
    import type {TabParamList} from './params';
    import TabBar from './TabBar.svelte';

    const Tabs = createBottomTabNavigator<TabParamList>();
</script>

{#snippet tabBar(props: TabBarProps)}
    <TabBar {...props} />
{/snippet}

<!-- Each tab owns its stack, so a detail page stays open behind a tab switch. -->
<Tabs.Navigator {tabBar}>
    <Tabs.Screen name="Photos" component={PhotosStack} />
    <Tabs.Screen name="Albums" component={AlbumsStack} />
    <Tabs.Screen name="Sync" component={SyncStack} />
    <Tabs.Screen name="Library" component={LibraryStack} />
</Tabs.Navigator>
