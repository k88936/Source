import {bundle} from '#lib/i18n/bundle';

export const dirPicker = bundle(
    {
        choose: 'Choose a folder',
        empty: 'No sub-folders here',
        emptyLibrary: 'No albums in the photo library',
        sync: (name: string) => `Sync "${name}"`,
        syncPrompt: 'Open a folder first',
        syncPromptLibrary: 'Choose an album first',
    },
    {
        choose: '选择一个目录',
        empty: '这个目录下没有子目录',
        emptyLibrary: '相册库里没有任何相册',
        sync: (name: string) => `同步 "${name}"`,
        syncPrompt: '请先进入一个目录',
        syncPromptLibrary: '请先选择一个相册',
    },
);
