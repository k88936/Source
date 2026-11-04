/**
 * Multi-select state for a grid.
 *
 * Shared by photos / album detail / search so selection mode, "select all" and
 * the action bar behave identically everywhere.
 */
export class Selection {
    active = $state(false);
    ids = $state<string[]>([]);

    get count(): number {
        return this.ids.length;
    }

    has(id: string): boolean {
        return this.ids.includes(id);
    }

    start(): void {
        this.active = true;
    }

    toggle(id: string): void {
        this.active = true;
        this.ids = this.ids.includes(id) ? this.ids.filter((item) => item !== id) : [...this.ids, id];
    }

    /** Toggle every id in `all`: if all are selected, clear; otherwise select all. */
    toggleAll(all: string[]): void {
        this.ids = all.every((id) => this.ids.includes(id)) ? [] : [...all];
    }

    clear(): void {
        this.ids = [];
    }

    exit(): void {
        this.active = false;
        this.ids = [];
    }

    /** Drop ids that are no longer in the grid (after a reload or a move). */
    retain(present: string[]): void {
        const keep = new Set(present);
        this.ids = this.ids.filter((id) => keep.has(id));
    }
}
