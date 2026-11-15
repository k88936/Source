import {toastManager} from '@immich/ui';
import {api, listen} from '#lib/api/client';
import type {AccountState} from '#lib/api/bindings/AccountState';
import type {JobRow} from '#lib/api/bindings/JobRow';
import type {SessionState} from '#lib/api/bindings/SessionState';
import type {Subscription} from 'subscription-client';
import {errorText} from '#lib/errors/i18n';
import {appMessages} from './app.i18n';
import {i18n} from '#lib/i18n/i18n.svelte';
import {jobsView} from '#lib/api/views';

export type ToastKind = 'success' | 'error' | 'info';

export class AppState {
    account = $state<AccountState | null>(null);
    /** What the client behind the account can do right now. */
    setup = $state<SessionState['setup']>('restoring');
    ready = $state(false);
    /** The queue: every job this process ran, newest first. */
    jobs = $state<JobRow[]>([]);
    columns = $state(3);
    private started = false;
    private queue: Subscription<JobRow> | null = null;

    async init(): Promise<void> {
        if (this.started) return;
        this.started = true;
        const stored = typeof localStorage !== 'undefined' ? localStorage.getItem('source-columns') : null;
        if (stored) {
            const n = Number(stored);
            if (n >= 2 && n <= 6) this.columns = n;
        }
        // The listeners come before the first read: the stored session is
        // checked in the background and its verdict arrives as an event, which
        // a read taken before subscribing would miss.
        await listen<SessionState>('session://setup', (payload) => {
            this.applySession(payload);
        });
        // The queue is a subscription like any other view, held for the life of
        // the window: a card reads a job's state from these rows instead of
        // asking again, and the queue only sends what changed.
        this.queue = await jobsView.watch({
            changed: () => {
                this.jobs = this.queue?.rows() ?? [];
            },
            failed: (error) => this.showToast(error, 'error'),
        });
        this.jobs = this.queue.rows();
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

    /** Read the row on disk and what the client behind it can do. */
    async syncSession(): Promise<void> {
        const state = await api.sessionState();
        this.applySession(state);
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
        const toast = {description: message};
        if (kind === 'error') {
            toastManager.danger(toast);
        } else if (kind === 'success') {
            toastManager.success(toast);
        } else {
            toastManager.info(toast);
        }
    }

    describeError(err: unknown): string {
        return errorText(err);
    }

    async logout(): Promise<void> {
        try {
            await api.logout();
        } finally {
            this.account = null;
        }
    }
}

export const app = new AppState();
