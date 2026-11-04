import {bundle} from '#lib/i18n/bundle';

export const queue = bundle(
    {
        title: 'Upload queue',
        retryAll: 'Retry all',
        items: (count: number) => (count === 1 ? '1 item' : `${count} items`),
        waitingAttempt: (attempt: number) => `Waiting for attempt ${attempt}`,
        waiting: 'Waiting',
        creatingSession: 'Opening session',
        completing: 'Finishing',
        failed: 'Failed',
    },
    {
        title: '上传队列',
        retryAll: '重试全部',
        items: (count: number) => `${count} 项`,
        waitingAttempt: (attempt: number) => `等待第 ${attempt} 次尝试`,
        waiting: '等待中',
        creatingSession: '创建会话',
        completing: '收尾中',
        failed: '失败',
    },
);
