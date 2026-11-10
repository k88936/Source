import type {CatalogChangedEvent} from '#lib/api/bindings/CatalogChangedEvent';

export type ChangeListener = (change: CatalogChangedEvent) => void;

/**
 * Fan-out for `catalog://changed`.
 *
 * Pages subscribe by kind instead of watching one global counter: an upload
 * completion carries the new tile so the timeline inserts it in place, while a
 * finished index pass asks everything to refetch.
 */
export class ChangeBus {
    private listeners = new Set<ChangeListener>();

    subscribe(listener: ChangeListener): () => void {
        this.listeners.add(listener);
        return () => {
            this.listeners.delete(listener);
        };
    }

    emit(change: CatalogChangedEvent): void {
        for (const listener of [...this.listeners]) listener(change);
    }
}

export const changes = new ChangeBus();
