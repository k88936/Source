import {bundle} from '#lib/i18n/bundle';

export const picker = bundle(
    {
        close: 'Close',
        filter: 'Filter albums',
        empty: 'No albums available',
        items: (count: number) => (count === 1 ? '1 item' : `${count} items`),
    },
    {
        close: '关闭',
        filter: '筛选相册',
        empty: '没有可用的相册',
        items: (count: number) => `${count} 项`,
    },
);
