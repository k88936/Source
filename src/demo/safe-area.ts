/**
 * Stands in for `@saurl/tauri-plugin-safe-area-insets-css-api` in the demo build.
 *
 * The plugin polls `__TAURI_INTERNALS__` for the window insets because it only
 * ever runs inside Tauri; the demo instead frames the app in a phone shell, so
 * it publishes the insets of that shell and lets the app's own `--safe-top` and
 * `--safe-bottom` keep the content clear of the notch and the home indicator.
 */
if (typeof document !== 'undefined') {
    document.documentElement.style.setProperty('--safe-area-inset-top', '54px');
    document.documentElement.style.setProperty('--safe-area-inset-bottom', '34px');
}

export {};
