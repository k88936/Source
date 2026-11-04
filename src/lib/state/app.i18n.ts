import {bundle} from '#lib/i18n/bundle';

/** Toasts raised by the app shell itself, outside any page. */
export const appMessages = bundle(
    {
        online: 'Connection restored',
        offline: 'Offline mode',
    },
    {
        online: '已恢复网络连接',
        offline: '已进入离线模式',
    },
);
