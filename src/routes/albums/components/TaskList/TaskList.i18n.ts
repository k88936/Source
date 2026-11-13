import {bundle} from '#lib/i18n/bundle';

export const taskList = bundle(
    {
        noTasks: 'Nothing queued',
        attempt: (count: number) => `attempt ${count}`,
        lastError: 'Last error',
        states: {
            queued: 'waiting',
            creatingSession: 'opening',
            uploading: 'uploading',
            completing: 'finishing',
            done: 'done',
            failed: 'failed',
            cancelled: 'cancelled',
        },
    },
    {
        noTasks: '没有排队的文件',
        attempt: (count: number) => `第 ${count} 次`,
        lastError: '最近错误',
        states: {
            queued: '等待',
            creatingSession: '建立会话',
            uploading: '上传中',
            completing: '收尾',
            done: '完成',
            failed: '失败',
            cancelled: '已取消',
        },
    },
);
