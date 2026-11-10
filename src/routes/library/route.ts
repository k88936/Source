import type {StackScreens} from 'svelte-navigation';
import type {TabParamList} from '../params';
import {viewerScreen} from '../../modals/Viewer/viewer-screen';
import Library from './Library.svelte';
import LocalAlbum from './LocalAlbums/LocalAlbum/LocalAlbum.svelte';
import LocalAlbums from './LocalAlbums/LocalAlbums.svelte';
import Settings from './Settings/Settings.svelte';

export type LibraryStackParamList = {
    Library: undefined;
    LocalAlbums: undefined;
    LocalAlbum: {id: string};
    Settings: undefined;
    Viewer: undefined;
};

export type LibraryNavParamList = LibraryStackParamList & TabParamList;

/**
 * Every screen of this stack, with the URL it answers on. The paths mirror the
 * on-device tree: library, then its local albums, then one album. The viewer is
 * a modal, so it takes no path and only adds a history entry.
 */
export const libraryScreens = {
    Library: {component: Library, path: '/library'},
    LocalAlbums: {component: LocalAlbums, path: '/library/local'},
    LocalAlbum: {component: LocalAlbum, path: '/library/local/:id'},
    Settings: {component: Settings, path: '/library/settings'},
    Viewer: viewerScreen,
} satisfies StackScreens<LibraryStackParamList>;
