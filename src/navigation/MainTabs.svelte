<script lang="ts">
    import {CloudSync, Image, Images, LibraryBig} from '@lucide/svelte';
    import {createBottomTabNavigator} from 'svelte-navigation';
    import {nav} from '#lib/i18n/nav.i18n';
    import {i18n} from '#lib/state/i18n.svelte';
    import AlbumsStack from './AlbumsStack.svelte';
    import LibraryStack from './LibraryStack.svelte';
    import PhotosStack from './PhotosStack.svelte';
    import SyncStack from './SyncStack.svelte';
    import type {TabParamList} from './params';

    const Tabs = createBottomTabNavigator<TabParamList>();
    const t = $derived(i18n.pick(nav));
</script>

<!-- Each tab owns its stack, so a detail page stays open behind a tab switch. -->
<Tabs.Navigator ariaLabel={t.ariaLabel}>
    <Tabs.Screen name="Photos" component={PhotosStack} options={{title: t.photos, icon: Image}} />
    <Tabs.Screen name="Albums" component={AlbumsStack} options={{title: t.albums, icon: Images}} />
    <Tabs.Screen name="Sync" component={SyncStack} options={{title: t.sync, icon: CloudSync}} />
    <Tabs.Screen name="Library" component={LibraryStack} options={{title: t.library, icon: LibraryBig}} />
</Tabs.Navigator>
