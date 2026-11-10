import type {Linking} from 'svelte-navigation';

/**
 * One path per stack screen. A screen without a path keeps the path of the
 * screen below, which is how the viewer is a modal: it takes a history entry
 * but not a URL. Tab roots are the tab names, so a URL always names the tab it
 * belongs to.
 */
export const linking: Linking = {
    '': {Welcome: '/welcome'},
    Photos: {Photos: '/photos'},
    Albums: {Albums: '/albums', Album: '/albums/:id'},
    Sync: {Sync: '/sync'},
    Library: {
        Library: '/library',
        LocalAlbums: '/library/local',
        LocalAlbum: '/library/local/:id',
        Settings: '/library/settings',
    },
};
