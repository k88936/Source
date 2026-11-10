import type {StackScreens} from 'svelte-navigation';
import type {TabParamList} from '../params';
import {viewerScreen} from '../../modals/Viewer/viewer-screen';
import Album from './Album/Album.svelte';
import Albums from './Albums.svelte';

export type AlbumsStackParamList = {
    Albums: undefined;
    Album: {id: string};
    Viewer: undefined;
};

export type AlbumsNavParamList = AlbumsStackParamList & TabParamList;

/**
 * Every screen of this stack, with the URL it answers on. The viewer is a modal,
 * so it takes no path: it keeps the album URL and only adds a history entry.
 */
export const albumsScreens = {
    Albums: {component: Albums, path: '/albums'},
    Album: {component: Album, path: '/albums/:id'},
    Viewer: viewerScreen,
} satisfies StackScreens<AlbumsStackParamList>;
