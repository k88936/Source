import {bundle} from '#lib/i18n/bundle';

export const localAlbum = bundle(
    {
        fallback: 'Local album',
        subtitle: (count: number, album: string) => `${count} items - synced to "${album}"`,
        refresh: 'Refresh folder',
        refreshed: 'Folder re-read',
        actions: 'Local album actions',
        noPhotos: 'No photos in this folder',
        unlinked: 'Sync removed',
        unlinkTitle: (name: string) => `Stop syncing "${name}"?`,
        unlinkMessage: 'Only the local record is removed; cloud files and albums are untouched.',
    },
    {
        fallback: '本机相册',
        subtitle: (count: number, album: string) => `${count} 项 - 同步到 "${album}"`,
        refresh: '刷新目录',
        refreshed: '目录已重新读取',
        actions: '本机相册操作',
        noPhotos: '这个目录里没有照片',
        unlinked: '已解除同步',
        unlinkTitle: (name: string) => `解除 "${name}" 的同步?`,
        unlinkMessage: '只会删掉本机的登记, 云端文件与相册不受影响.',
    },
);
