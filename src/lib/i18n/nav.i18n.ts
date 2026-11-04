import {bundle} from './bundle';

/** The five destinations, shared by the tab bar and the sidebar. */
export const nav = bundle(
    {
        albums: 'Albums',
        ariaLabel: 'Main navigation',
        library: 'Library',
        photos: 'Photos',
        search: 'Search',
        settings: 'Settings',
    },
    {
        albums: '相册',
        ariaLabel: '主导航',
        library: '资料库',
        photos: '照片',
        search: '搜索',
        settings: '设置',
    },
);
