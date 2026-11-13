import type {StackScreens} from 'svelte-navigation';
import type {TabParamList} from '../params';
import {viewerScreen} from '../../modals/Viewer/viewer-screen';
import Albums from './Albums.svelte';
import Album from './Album/Album.svelte';
import LocalAlbum from './LocalAlbum/LocalAlbum.svelte';

export type AlbumsStackParamList = {
    Albums: undefined;
    Album: {id: string};
    LocalAlbum: {id: string};
    Viewer: undefined;
};

export type AlbumsNavParamList = AlbumsStackParamList & TabParamList;

/**
 * The graph hands a node over to the page that owns it: the album and local
 * album screens are declared here so one opened from the graph keeps a back
 * button pointing at it. The viewer is a modal, so it takes no path.
 */
export const albumsScreens = {
    Albums: {component: Albums, path: '/albums'},
    Album: {component: Album, path: '/albums/album/:id'},
    LocalAlbum: {component: LocalAlbum, path: '/albums/local/:id'},
    Viewer: viewerScreen,
} satisfies StackScreens<AlbumsStackParamList>;
