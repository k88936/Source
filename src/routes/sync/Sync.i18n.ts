import {bundle} from '#lib/i18n/bundle';

export const sync = bundle(
    {
        empty: 'No albums yet',
        attached: (name: string, album: string) => `"${name}" now syncs to "${album}"`,
        unbound: 'Sync removed',
        stopTitle: (name: string) => `Stop syncing "${name}"?`,
        stopMessage: 'Only the local record is removed; cloud files and albums are untouched.',
    },
    {
        empty: '还没有相册',
        attached: (name: string, album: string) => `"${name}" 已接入 "${album}"`,
        unbound: '已解除映射',
        stopTitle: (name: string) => `解除 "${name}" 的同步?`,
        stopMessage: '只会删掉本机的登记, 云端文件与相册不受影响.',
    },
);
