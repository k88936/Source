import {bundle} from '#lib/i18n/bundle';

export const albums = bundle(
    {
        title: 'Albums',
        count: (count: number) => (count === 1 ? '1 album' : `${count} albums`),
        refreshAll: 'Refresh all albums',
        addAlbum: 'Add album',
        failed: 'Unable to load albums',
        empty: 'No albums yet',
        hint: 'An album is a folder in the cloud - it only lists the photos and videos directly inside it',
        added: (name: string) => `Added album "${name}"`,
        refreshed: 'Albums refreshed',
        adding: 'Adding album...',
    },
    {
        title: '相册',
        count: (count: number) => `${count} 个相册`,
        refreshAll: '刷新全部相册',
        addAlbum: '添加相册',
        failed: '无法加载相册',
        empty: '还没有相册',
        hint: '相册就是云端的一个文件夹 - 只收录它直接包含的照片与视频',
        added: (name: string) => `已添加相册 "${name}"`,
        refreshed: '相册已刷新',
        adding: '正在添加相册...',
    },
);
