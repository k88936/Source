import {bundle} from '#lib/i18n/bundle';

export const card = bundle(
    {
        items: (count: number, path: string) => `${count} items - ${path}`,
    },
    {
        items: (count: number, path: string) => `${count} 项 - ${path}`,
    },
);
