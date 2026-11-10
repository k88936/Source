import {bundle} from '#lib/i18n/bundle';

export const library = bundle(
    {
        sync: 'Sync now',
        syncing: (visited: number, total: number) => `Syncing ${visited} of ${total}`,
        syncFailed: (count: number) => `${count} files could not be queued`,
        syncDone: 'Sync finished',
        synced: (albums: number, photos: number) => `${albums} synced - ${photos} photos`,
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
        sync: '立即同步',
        syncing: (visited: number, total: number) => `正在同步 ${visited}/${total}`,
        syncFailed: (count: number) => `${count} 个文件未能加入上传队列`,
        syncDone: '同步完成',
        synced: (albums: number, photos: number) => `${albums} 个已同步 - ${photos} 张照片`,
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
