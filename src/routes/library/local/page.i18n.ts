import {bundle} from '#lib/i18n/bundle';

export const local = bundle(
    {
        title: 'Local albums',
        synced: (name: string, album: string) => `"${name}" synced to "${album}"`,
        empty: 'No local albums synced yet',
        emptyHint: 'Pick a local folder to sync one way to Cloudreve',
        items: (count: number, album: string) => `${count} items - synced to "${album}"`,
        syncTo: 'Sync to',
    },
    {
        title: '本机相册',
        synced: (name: string, album: string) => `"${name}" 已同步到 "${album}"`,
        empty: '还没有同步的本机相册',
        emptyHint: '选择本地相册单向同步到Cloudreve',
        items: (count: number, album: string) => `${count} 项 - 同步到 "${album}"`,
        syncTo: '同步到',
    },
);
