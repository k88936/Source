import {api, listen} from '#lib/api/client';
import type {CatalogChangedEvent} from '#lib/api/bindings/CatalogChangedEvent';
import type {AccountState} from '#lib/api/bindings/AccountState';
import type {Job} from '#lib/api/bindings/Job';
import type {JobKind} from '#lib/api/bindings/JobKind';
import type {SessionState} from '#lib/api/bindings/SessionState';
import {errorText} from '#lib/errors/i18n';
import {appMessages} from './app.i18n';
import {changes} from './changes';
import {i18n} from './i18n.svelte';

export type ToastKind = 'success' | 'error' | 'info';

export class AppState {
    account = $state<AccountState | null>(null);
    /** What the client behind the account can do right now. */
    setup = $state<SessionState['setup']>('restoring');
    ready = $state(false);
    /** The queue's board: every job this process ran, newest first. */
    jobs = $state<Job<JobKind>[]>([]);
    /** Whether background work is paused, persisted by the adapter. */
    jobsPaused = $state(false);
    columns = $state(3);
    toast = $state<{message: string; kind: ToastKind} | null>(null);
    private started = false;

    async init(): Promise<void> {
        if (this.started) return;
        this.started = true;
        const stored = typeof localStorage !== 'undefined' ? localStorage.getItem('source-columns') : null;
        if (stored) {
            const n = Number(stored);
            if (n >= 2 && n <= 6) this.columns = n;
        }
        try {
            this.jobsPaused = await api.jobsPaused();
        } catch (err) {
            this.showToast(this.describeError(err), 'error');
        }
        // The listeners come before the first poll: the stored session is
        // checked in the background and its verdict arrives as an event, which
        // a poll taken before subscribing would miss.
        await listen<SessionState>('session://setup', (payload) => {
            this.applySession(payload);
            if (payload.setup === 'live') void this.afterLogin();
        });
        await listen<CatalogChangedEvent>('catalog://changed', (payload) => {
            changes.emit(payload);
        });
        await listen<Job<JobKind>[]>('job://changed', (payload) => {
            this.jobs = payload;
        });
        try {
            await this.syncSession();
        } catch (err) {
            this.describeError(err);
        } finally {
            this.ready = true;
        }
        if (typeof window !== 'undefined') {
            window.addEventListener('online', () => this.showToast(i18n.pick(appMessages).online, 'success'));
            window.addEventListener('offline', () => this.showToast(i18n.pick(appMessages).offline, 'info'));
        }
    }

    async afterLogin(): Promise<void> {
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

    /** Read the row on disk and what the client behind it can do. */
    async syncSession(): Promise<void> {
        const state = await api.sessionState();
        this.applySession(state);
        if (state.account) await this.afterLogin();
    }

    applySession(state: SessionState): void {
        this.account = state.account;
        this.setup = state.setup;
    }

    async refreshAccount(): Promise<void> {
        await this.syncSession();
    }

    setColumns(columns: number): void {
        this.columns = Math.min(6, Math.max(2, columns));
        if (typeof localStorage !== 'undefined') {
            localStorage.setItem('source-columns', String(this.columns));
        }
    }

    showToast(message: string, kind: ToastKind = 'info'): void {
        this.toast = {message, kind};
        setTimeout(() => {
            if (this.toast?.message === message) this.toast = null;
        }, 3200);
    }

    describeError(err: unknown): string {
        return errorText(err);
    }

    async logout(): Promise<void> {
        try {
            await api.logout();
        } finally {
            this.account = null;
            this.jobs = [];
        }
    }
}

export const app = new AppState();
