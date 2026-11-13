import {bundle} from '#lib/i18n/bundle';

export const sync = bundle(
    {
        localAlbums: 'Local albums',
        cloudAlbums: 'Cloud albums',
        attached: (name: string, album: string) => `"${name}" now syncs to "${album}"`,
        added: (name: string) => `Added album "${name}"`,
        adding: 'Adding album...',
        unbound: 'Sync removed',
        stopTitle: (name: string) => `Stop syncing "${name}"?`,
        stopMessage: 'Only the local record is removed; cloud files and albums are untouched.',
    },
    {
        localAlbums: '本地相册',
        cloudAlbums: '云端相册',
        attached: (name: string, album: string) => `"${name}" 已接入 "${album}"`,
        added: (name: string) => `已添加相册 "${name}"`,
        adding: '正在添加相册...',
        unbound: '已解除映射',
        stopTitle: (name: string) => `解除 "${name}" 的同步?`,
        stopMessage: '只会删掉本机的登记, 云端文件与相册不受影响.',
    },
);
