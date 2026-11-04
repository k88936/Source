import {bundle} from '#lib/i18n/bundle';

export const picker = bundle(
    {
        moveHint: 'A Cloudreve album is a folder, so moving takes the files out of their current folder.',
        close: 'Close',
        filter: 'Filter albums',
        empty: 'No albums available',
        items: (count: number) => (count === 1 ? '1 item' : `${count} items`),
    },
    {
        moveHint: 'Cloudreve 的相册就是文件夹, 移动后文件会离开原目录.',
        close: '关闭',
        filter: '筛选相册',
        empty: '没有可用的相册',
        items: (count: number) => `${count} 项`,
    },
);
