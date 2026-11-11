import js from '@eslint/js';
import prettier from 'eslint-config-prettier';
import svelte from 'eslint-plugin-svelte';
import globals from 'globals';
import ts from 'typescript-eslint';

export default [
    {
        ignores: [
            'build/',
            'dist/',
            'dist-demo/',
            'site/dist/',
            '.svelte-kit/',
            '**/target/',
            'tmp/',
            'src-tauri/gen/',
            'src/lib/api/bindings/',
            'src/service-worker/',
        ],
    },
    js.configs.recommended,
    ...ts.configs.recommended,
    ...svelte.configs.recommended,
    {
        languageOptions: {
            globals: {...globals.browser, ...globals.node},
        },
        rules: {
            '@typescript-eslint/no-unused-vars': ['error', {argsIgnorePattern: '^_', varsIgnorePattern: '^_'}],
            // Plain Map/Set are used for imperative registries and local helpers, where reactivity is not wanted.
            'svelte/prefer-svelte-reactivity': 'off',
        },
    },
    {
        files: ['**/*.svelte', '**/*.svelte.ts', '**/*.svelte.js'],
        languageOptions: {
            parserOptions: {
                parser: ts.parser,
            },
        },
    },
    prettier,
    ...svelte.configs.prettier,
];
