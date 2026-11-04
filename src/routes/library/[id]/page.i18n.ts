import {bundle} from '#lib/i18n/bundle';

export const localAlbum = bundle(
    {
        fallback: 'Local album',
        selected: (count: number) => `${count} selected`,
        subtitle: (count: number, album: string) => `${count} items - synced to "${album}"`,
        cancelSelection: 'Cancel selection',
        unlink: 'Stop syncing',
        noPhotos: 'No photos in this folder',
        choosePhotos: 'Choose photos to upload',
        uploadCreated: 'Upload queued, check progress in Library',
        unlinked: 'Sync removed',
        selectedCount: (count: number) => `${count} selected`,
        uploadToAlbum: 'Upload to album',
        unlinkTitle: (name: string) => `Stop syncing "${name}"?`,
        unlinkMessage: 'Only the local record is removed; cloud files and albums are untouched.',
    },
    {
        fallback: '本机相册',
        selected: (count: number) => `已选择 ${count} 项`,
        subtitle: (count: number, album: string) => `${count} 项 - 同步到 "${album}"`,
        cancelSelection: '取消选择',
        unlink: '解除同步',
        noPhotos: '这个目录里没有照片',
        choosePhotos: '选择要上传的照片',
        uploadCreated: '上传任务已创建, 可在资料库查看进度',
        unlinked: '已解除同步',
        selectedCount: (count: number) => `已选 ${count} 项`,
        uploadToAlbum: '上传到相册',
        unlinkTitle: (name: string) => `解除 "${name}" 的同步?`,
        unlinkMessage: '只会删掉本机的登记, 云端文件与相册不受影响.',
    },
);
