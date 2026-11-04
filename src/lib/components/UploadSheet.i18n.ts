import {bundle} from '#lib/i18n/bundle';

export const upload = bundle(
    {
        title: 'Upload to album',
        subtitle: 'Pick photos from this device - uploaded in full quality',
        enqueuing: 'Adding to queue...',
        needAlbum: 'Pick an album first',
        uploadLabel: (count: number, album: string) => `Upload ${count} items to ${album}`,
        pickAlbum: 'Choose album',
        queued: (count: number, album: string) => `Added ${count} items to the queue -> ${album}`,
        empty: 'No local photos to upload',
        emptyHint: 'Sync a local album in Library first',
        destination: 'Upload to',
    },
    {
        title: '上传到相册',
        subtitle: '选择本机照片 - 原图上传',
        enqueuing: '正在加入队列...',
        needAlbum: '先选择一个相册',
        uploadLabel: (count: number, album: string) => `上传 ${count} 项到 ${album}`,
        pickAlbum: '选择相册',
        queued: (count: number, album: string) => `已加入上传队列 - ${count} 项 -> ${album}`,
        empty: '没有可上传的本机照片',
        emptyHint: '先在资料库里同步一个本机相册',
        destination: '上传到',
    },
);
