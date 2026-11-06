import {api, listen} from '#lib/api/client';
import type {CatalogChangedEvent} from '#lib/api/bindings/CatalogChangedEvent';
import type {AccountState} from '#lib/api/bindings/AccountState';
import type {UploadTask} from '#lib/api/bindings/UploadTask';
import type {Job} from '#lib/api/bindings/Job';
import {appMessages} from './app.i18n';
import {changes} from './changes';
import {i18n} from './i18n.svelte';

export type ToastKind = 'success' | 'error' | 'info';

export class AppState {
    account = $state<AccountState | null>(null);
    ready = $state(false);
    uploads = $state<UploadTask[]>([]);
    /** The queue's board: every job this process ran, newest first. */
    jobs = $state<Job[]>([]);
    /** Whether background work is paused, persisted by the adapter. */
    jobsPaused = $state(false);
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
            this.jobsPaused = await api.jobsPaused();
        } catch (err) {
            this.showToast(this.describeError(err), 'error');
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
        await listen<Job[]>('job://changed', (payload) => {
            this.jobs = payload;
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
        await this.refreshJobs();
    }

    /** Pick up jobs that were already queued when the window loaded. */
    async refreshJobs(): Promise<void> {
        this.jobs = await api.jobList();
    }

    /** Pause or resume every background job, and remember the choice. */
    async setJobsPaused(paused: boolean): Promise<void> {
        const previous = this.jobsPaused;
        this.jobsPaused = paused;
        try {
            await api.setJobsPaused({paused});
        } catch (err) {
            this.jobsPaused = previous;
            this.showToast(this.describeError(err), 'error');
        }
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
