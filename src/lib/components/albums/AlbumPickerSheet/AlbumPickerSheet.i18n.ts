import {bundle} from '#lib/i18n/bundle';

export const picker = bundle(
    {
        filter: 'Filter albums',
        empty: 'No albums available',
        items: (count: number) => (count === 1 ? '1 item' : `${count} items`),
    },
    {
        filter: '筛选相册',
        empty: '没有可用的相册',
        items: (count: number) => `${count} 项`,
    },
);
