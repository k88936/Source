import {bundle} from '#lib/i18n/bundle';

export const viewer = bundle(
    {
        favorite: 'Favorite',
        info: 'Info',
        mediaViewer: 'Media viewer',
        previous: 'Previous',
        next: 'Next',
        exported: (count: number, directory: string) => `Exported ${count} items to ${directory}`,
    },
    {
        favorite: '收藏',
        info: '信息',
        mediaViewer: '媒体查看器',
        previous: '上一张',
        next: '下一张',
        exported: (count: number, directory: string) => `已导出 ${count} 项到 ${directory}`,
    },
);
