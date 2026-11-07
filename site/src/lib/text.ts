import type {Locale} from './i18n.svelte';

interface Text {
    credit: string;
    switchLabel: string;
    previewTitle: string;
    dark: string;
    light: string;
}

export const text: Record<Locale, Text> = {
    zh: {
        credit: '图片来自 Bing 每日一图',
        switchLabel: 'EN',
        previewTitle: 'Source 应用预览',
        dark: '深色',
        light: '浅色',
    },
    en: {
        credit: 'Images from Bing daily photos',
        switchLabel: '中文',
        previewTitle: 'Source app preview',
        dark: 'Dark',
        light: 'Light',
    },
};
