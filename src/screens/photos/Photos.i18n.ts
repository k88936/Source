import {bundle} from '#lib/i18n/bundle';

export const photos = bundle(
    {
        title: 'Photos',
        selected: (count: number) => `${count} selected`,
        exitSelection: 'Exit selection',
        failed: 'Failed to load',
        empty: 'The timeline is still empty',
        addAlbum: 'Add album',
        exported: (count: number) => `Exported ${count} items`,
        moved: (count: number, album: string) => `Moved ${count} items to "${album}"`,
    },
    {
        title: '照片',
        selected: (count: number) => `已选择 ${count} 项`,
        exitSelection: '退出选择',
        failed: '加载失败',
        empty: '时间线还是空的',
        addAlbum: '添加相册',
        exported: (count: number) => `已导出 ${count} 项`,
        moved: (count: number, album: string) => `已移动 ${count} 项到 "${album}"`,
    },
);
