import {bundle} from '#lib/i18n/bundle';

export const dirPicker = bundle(
    {
        title: 'Sync local album',
        choose: 'Choose a folder',
        empty: 'No sub-folders here',
        sync: (name: string) => `Sync "${name}"`,
        syncPrompt: 'Open a folder first',
    },
    {
        title: '同步本机相册',
        choose: '选择一个目录',
        empty: '这个目录下没有子目录',
        sync: (name: string) => `同步 "${name}"`,
        syncPrompt: '请先进入一个目录',
    },
);
