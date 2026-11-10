import type {TabScreens} from 'svelte-navigation';
import type {TabParamList} from './params';
import AlbumsStack from './albums/Stack.svelte';
import {albumsScreens} from './albums/route';
import LibraryStack from './library/Stack.svelte';
import {libraryScreens} from './library/route';
import PhotosStack from './photos/Stack.svelte';
import {photosScreens} from './photos/route';
import SyncStack from './sync/Stack.svelte';
import {syncScreens} from './sync/route';

/**
 * The tabs in display order, each with its stack and the screens that stack
 * answers on. The navigator reads the tab list, the stacks and the URLs from
 * these entries, so a tab cannot be listed twice and drift.
 */
export const tabs = {
    Photos: {component: PhotosStack, screens: photosScreens},
    Albums: {component: AlbumsStack, screens: albumsScreens},
    Sync: {component: SyncStack, screens: syncScreens},
    Library: {component: LibraryStack, screens: libraryScreens},
} satisfies TabScreens<TabParamList>;
