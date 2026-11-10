import {bundle} from '#lib/i18n/bundle';

export const albums = bundle(
    {
        title: 'Albums',
        count: (count: number) => (count === 1 ? '1 album' : `${count} albums`),
        refreshAll: 'Refresh all albums',
        stopRefresh: 'Stop refreshing',
        progress: (visited: number, total: number) => `Refreshing ${visited} of ${total}`,
        partialFailed: (count: number) => `${count} albums could not be refreshed`,
        addAlbum: 'Add album',
        failed: 'Unable to load albums',
        empty: 'No albums yet',
        added: (name: string) => `Added album "${name}"`,
        refreshed: 'Albums refreshed',
        adding: 'Adding album...',
    },
    {
        title: '相册',
        count: (count: number) => `${count} 个相册`,
        refreshAll: '刷新全部相册',
        stopRefresh: '停止刷新',
        progress: (visited: number, total: number) => `已刷新 ${visited}/${total}`,
        partialFailed: (count: number) => `${count} 个相册刷新失败`,
        addAlbum: '添加相册',
        failed: '无法加载相册',
        empty: '还没有相册',
        added: (name: string) => `已添加相册 "${name}"`,
        refreshed: '相册已刷新',
        adding: '正在添加相册...',
    },
);
