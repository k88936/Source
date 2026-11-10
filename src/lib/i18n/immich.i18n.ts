import {bundle} from './bundle';

/**
 * Text `@immich/ui` shows on its own: tooltips, modal defaults and toast titles.
 * The keys line up with the package's `defaultTranslations`, so this bundle only
 * covers what this app can actually reach.
 */
export const immich = bundle(
    {
        hidePassword: 'Hide password',
        openCalendar: 'Open calendar',
        openMenu: 'Open menu',
        promptDefault: 'Are you sure you want to do this?',
        searchNoResults: 'No results',
        searchPlaceholder: 'Search',
        showPassword: 'Show password',
        toastDanger: 'Error',
        toastInfo: 'Info',
        toastSuccess: 'Success',
        toastWarning: 'Warning',
        toggleDarkTheme: 'Toggle dark theme',
    },
    {
        hidePassword: '隐藏密码',
        openCalendar: '选择日期',
        openMenu: '打开菜单',
        promptDefault: '确定要执行此操作吗?',
        searchNoResults: '没有结果',
        searchPlaceholder: '搜索',
        showPassword: '显示密码',
        toastDanger: '错误',
        toastInfo: '提示',
        toastSuccess: '成功',
        toastWarning: '警告',
        toggleDarkTheme: '切换深色主题',
    },
);
