import {invoke} from '@tauri-apps/api/core';
import {listen} from '@tauri-apps/api/event';

export interface Transport {
    invoke<T>(command: string, params?: Record<string, unknown>): Promise<T>;
    listen<T>(event: string, handler: (payload: T) => void): Promise<() => void>;
}

/** The app runs only inside Tauri, so the IPC bridge is the transport. */
export const transport: Transport = {
    invoke: <T>(command: string, params?: Record<string, unknown>) => invoke<T>(command, params ? {params} : undefined),
    listen: <T>(event: string, handler: (payload: T) => void) => listen<T>(event, (e) => handler(e.payload)),
};
