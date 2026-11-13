import {bundle} from '#lib/i18n/bundle';

export const remotePicker = bundle(
    {
        current: (path: string) => `Current: ${path}`,
        newFolder: 'New folder',
        folderName: 'Folder name',
        create: 'Create',
        empty: 'No sub-folders here',
        add: (name: string) => `Add "${name}" as album`,
    },
    {
        current: (path: string) => `当前: ${path}`,
        newFolder: '新建文件夹',
        folderName: '新文件夹名',
        create: '创建',
        empty: '这个目录下没有子文件夹',
        add: (name: string) => `添加 "${name}" 为相册`,
    },
);
