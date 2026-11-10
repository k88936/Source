import type {StackScreens} from 'svelte-navigation';
import type {TabParamList} from '../params';
import {viewerScreen} from '../../modals/Viewer/viewer-screen';
import Photos from './Photos.svelte';

export type PhotosStackParamList = {
    Photos: undefined;
    Viewer: undefined;
};

export type PhotosNavParamList = PhotosStackParamList & TabParamList;

/**
 * Every screen of this stack, with the URL it answers on. The viewer is a modal,
 * so it takes no path: it keeps the photos URL and only adds a history entry.
 */
export const photosScreens = {
    Photos: {component: Photos, path: '/photos'},
    Viewer: viewerScreen,
} satisfies StackScreens<PhotosStackParamList>;
