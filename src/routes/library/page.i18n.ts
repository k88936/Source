import {bundle} from '#lib/i18n/bundle';

export const library = bundle(
    {
        title: 'Library',
        localAlbums: 'Local albums',
        syncPrompt: 'Sync a local folder to a cloud album',
        synced: (albums: number, photos: number) => `${albums} synced - ${photos} photos`,
        more: 'More',
        notOpen: 'Not available yet',
        upcoming: {
            favorites: 'Favorites',
            archive: 'Archive',
            sharedLinks: 'Shared links',
            trash: 'Trash',
            people: 'People',
            places: 'Places',
            memories: 'Memories',
            cloudFolders: 'Cloud folders',
            lockedFolders: 'Locked folders',
            partners: 'Partners',
        },
    },
    {
        title: '资料库',
        localAlbums: '本机相册',
        syncPrompt: '把一个本机目录同步到云端相册',
        synced: (albums: number, photos: number) => `${albums} 个已同步 - ${photos} 张照片`,
        more: '更多',
        notOpen: '尚未开放',
        upcoming: {
            favorites: '收藏',
            archive: '归档',
            sharedLinks: '共享链接',
            trash: '回收站',
            people: '人物',
            places: '地点',
            memories: '回忆',
            cloudFolders: '云端文件夹',
            lockedFolders: '锁定文件夹',
            partners: '伙伴',
        },
    },
);
