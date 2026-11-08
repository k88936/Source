import {bundle} from '#lib/i18n/bundle';

export const queue = bundle(
    {
        title: 'Job queue',
        items: (count: number) => (count === 1 ? '1 job' : `${count} jobs`),
        kind: {
            index: 'Album index',
            sync: 'Directory scan',
            upload: 'Upload',
        },
        state: {
            queued: 'Queued',
            running: 'Running',
            paused: 'Paused',
            done: 'Done',
            failed: 'Failed',
            cancelled: 'Cancelled',
        },
        group: (live: number, done: number, failed: number, cancelled: number) =>
            `${live} live - ${done} done - ${failed} failed` + (cancelled > 0 ? ` - ${cancelled} cancelled` : ''),
        processed: (count: number) => `${count} processed`,
        more: (count: number) => `and ${count} more`,
        empty: 'No jobs yet',
    },
    {
        title: '任务队列',
        items: (count: number) => `${count} 个任务`,
        kind: {
            index: '相册索引',
            sync: '目录扫描',
            upload: '上传',
        },
        state: {
            queued: '排队中',
            running: '运行中',
            paused: '已暂停',
            done: '完成',
            failed: '失败',
            cancelled: '已取消',
        },
        group: (live: number, done: number, failed: number, cancelled: number) =>
            `${live} 进行中 - ${done} 完成 - ${failed} 失败` + (cancelled > 0 ? ` - ${cancelled} 已取消` : ''),
        processed: (count: number) => `已处理 ${count}`,
        more: (count: number) => `还有 ${count} 项`,
        empty: '暂无任务',
    },
);
