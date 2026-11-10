export type TabParamList = {
    Photos: undefined;
    Albums: undefined;
    Sync: undefined;
    Library: undefined;
};

export type PhotosStackParamList = {
    Photos: undefined;
    Viewer: undefined;
};

export type AlbumsStackParamList = {
    Albums: undefined;
    Album: {id: string};
    Viewer: undefined;
};

export type SyncStackParamList = {
    Sync: undefined;
};

export type LibraryStackParamList = {
    Library: undefined;
    LocalAlbums: undefined;
    LocalAlbum: {id: string};
    Settings: undefined;
    Viewer: undefined;
};

export type WelcomeStackParamList = {
    Welcome: undefined;
};

/**
 * What a screen may navigate to: the screens of its own stack, plus the tab
 * names so a page can hand over to another tab.
 */
export type PhotosNavParamList = PhotosStackParamList & TabParamList;
export type AlbumsNavParamList = AlbumsStackParamList & TabParamList;
export type SyncNavParamList = SyncStackParamList & TabParamList;
export type LibraryNavParamList = LibraryStackParamList & TabParamList;
