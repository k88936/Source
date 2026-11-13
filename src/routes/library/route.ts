import type {StackScreens} from 'svelte-navigation';
import type {TabParamList} from '../params';
import {viewerScreen} from '../../modals/Viewer/viewer-screen';
import Library from './Library.svelte';
import Settings from './Settings/Settings.svelte';

export type LibraryStackParamList = {
    Library: undefined;
    Settings: undefined;
    Viewer: undefined;
};

export type LibraryNavParamList = LibraryStackParamList & TabParamList;

/**
 * Every screen of this stack, with the URL it answers on. Library is the page
 * itself and Settings is its one drill-down. The viewer is a modal, so it takes
 * no path and only adds a history entry.
 */
export const libraryScreens = {
    Library: {component: Library, path: '/library'},
    Settings: {component: Settings, path: '/library/settings'},
    Viewer: viewerScreen,
} satisfies StackScreens<LibraryStackParamList>;
