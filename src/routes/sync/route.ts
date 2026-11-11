import type {StackScreens} from 'svelte-navigation';
import type {TabParamList} from '../params';
import {viewerScreen} from '../../modals/Viewer/viewer-screen';
import Album from '../albums/Album/Album.svelte';
import LocalAlbum from '../library/LocalAlbums/LocalAlbum/LocalAlbum.svelte';
import Sync from './Sync.svelte';

export type SyncStackParamList = {
    Sync: undefined;
    Album: {id: string};
    LocalAlbum: {id: string};
    Viewer: undefined;
};

export type SyncNavParamList = SyncStackParamList & TabParamList;

/**
 * The graph hands a node over to the page that owns it: the album and local
 * album screens are the same components the other tabs use, declared here so
 * one opened from the graph keeps a back button pointing at it. The viewer is a
 * modal, so it takes no path.
 */
export const syncScreens = {
    Sync: {component: Sync, path: '/sync'},
    Album: {component: Album, path: '/sync/album/:id'},
    LocalAlbum: {component: LocalAlbum, path: '/sync/local/:id'},
    Viewer: viewerScreen,
} satisfies StackScreens<SyncStackParamList>;
