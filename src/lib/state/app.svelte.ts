import {api, listen} from '#lib/api/client';
import type {CatalogChangedEvent} from '#lib/api/bindings/CatalogChangedEvent';
import type {AccountState} from '#lib/api/bindings/AccountState';
import type {UploadTask} from '#lib/api/bindings/UploadTask';
import {appMessages} from './app.i18n';
import {changes} from './changes';
import {i18n} from './i18n.svelte';

export type ToastKind = 'success' | 'error' | 'info';

export class AppState {
    account = $state<AccountState | null>(null);
    ready = $state(false);
    uploads = $state<UploadTask[]>([]);
    columns = $state(3);
    toast = $state<{message: string; kind: ToastKind} | null>(null);
    private started = false;

    async init(): Promise<void> {
        if (this.started) return;
        this.started = true;
        const stored = typeof localStorage !== 'undefined' ? localStorage.getItem('album-columns') : null;
        if (stored) {
            const n = Number(stored);
            if (n >= 2 && n <= 6) this.columns = n;
        }
        try {
            this.account = await api.accountState();
            if (this.account) {
                await this.afterLogin();
            }
        } catch (err) {
            this.describeError(err);
        } finally {
            this.ready = true;
        }

        await listen<CatalogChangedEvent>('catalog://changed', (payload) => {
            changes.emit(payload);
        });
        await listen<{taskId: string}>('upload://state', () => {
            void this.refreshUploads();
        });
        await listen<{taskId: string; uploaded: number; total: number}>('upload://progress', (payload) => {
            const task = this.uploads.find((t) => t.id === payload.taskId);
            if (task) {
                task.uploaded = payload.uploaded;
                task.total = payload.total;
            }
        });
        if (typeof window !== 'undefined') {
            window.addEventListener('online', () => this.showToast(i18n.pick(appMessages).online, 'success'));
            window.addEventListener('offline', () => this.showToast(i18n.pick(appMessages).offline, 'info'));
        }
    }

    async afterLogin(): Promise<void> {
        await this.refreshUploads();
    }

    async refreshAccount(): Promise<void> {
        this.account = await api.accountState();
    }

    async refreshUploads(): Promise<void> {
        try {
            this.uploads = await api.uploadTasks();
        } catch {
            // uploads are best-effort on old servers
        }
    }

    async retryUpload(taskId: string): Promise<void> {
        try {
            await api.retryUpload({taskId});
            await this.refreshUploads();
        } catch (err) {
            this.showToast(this.describeError(err), 'error');
        }
    }

    setColumns(columns: number): void {
        this.columns = Math.min(6, Math.max(2, columns));
        if (typeof localStorage !== 'undefined') {
            localStorage.setItem('album-columns', String(this.columns));
        }
    }

    showToast(message: string, kind: ToastKind = 'info'): void {
        this.toast = {message, kind};
        setTimeout(() => {
            if (this.toast?.message === message) this.toast = null;
        }, 3200);
    }

    describeError(err: unknown): string {
        if (err && typeof err === 'object' && 'message' in err) {
            return String((err as {message: unknown}).message);
        }
        return String(err);
    }

    async logout(): Promise<void> {
        try {
            await api.logout();
        } finally {
            this.account = null;
            this.uploads = [];
        }
    }
}

export const app = new AppState();
