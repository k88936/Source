import type { AssetSummary } from "#lib/api/bindings/AssetSummary";
import type { TimelineCursor } from "#lib/api/bindings/TimelineCursor";
import type { TimelinePage } from "#lib/api/bindings/TimelinePage";
import { app } from "./app.svelte";

/** Page size used by every timeline-style list. */
export const PAGE_SIZE = 80;

export type PageLoader = (
  cursor: TimelineCursor | null,
  limit: number,
) => Promise<TimelinePage>;

/**
 * Cursor-paged, newest-first asset list shared by photos / album / search.
 *
 * Pages only own their query (`loader`) and selection; paging, error state and
 * patching live here so the three lists cannot drift apart.
 */
export class Timeline {
  items = $state<AssetSummary[]>([]);
  next = $state<TimelineCursor | null>(null);
  loading = $state(false);
  loaded = $state(false);
  error = $state("");

  private loader: PageLoader;
  private requestId = 0;

  constructor(loader: PageLoader) {
    this.loader = loader;
  }

  /** Swap the query, e.g. when the album or the search filters change. */
  setLoader(loader: PageLoader): void {
    this.loader = loader;
  }

  async reload(): Promise<void> {
    const id = ++this.requestId;
    this.loading = true;
    this.error = "";
    try {
      const page = await this.loader(null, PAGE_SIZE);
      if (id !== this.requestId) return;
      this.items = page.items;
      this.next = page.next;
      this.loaded = true;
    } catch (err) {
      if (id !== this.requestId) return;
      this.error = app.describeError(err);
      this.loaded = true;
    } finally {
      if (id === this.requestId) this.loading = false;
    }
  }

  async loadMore(): Promise<void> {
    if (this.loading || !this.next) return;
    const id = this.requestId;
    this.loading = true;
    try {
      const page = await this.loader(this.next, PAGE_SIZE);
      if (id !== this.requestId) return;
      const known = new Set(this.items.map((item) => item.id));
      this.items = [
        ...this.items,
        ...page.items.filter((item) => !known.has(item.id)),
      ];
      this.next = page.next;
    } catch (err) {
      if (id === this.requestId) this.error = app.describeError(err);
    } finally {
      if (id === this.requestId) this.loading = false;
    }
  }

  /** Insert or replace one asset, keeping the newest-first order. */
  upsert(asset: AssetSummary): void {
    const index = this.items.findIndex((item) => item.id === asset.id);
    if (index >= 0) {
      this.items[index] = asset;
      return;
    }
    const at = this.items.findIndex((item) => isNewer(asset, item));
    if (at < 0) this.items.push(asset);
    else this.items.splice(at, 0, asset);
  }

  remove(ids: string[]): void {
    const drop = new Set(ids);
    this.items = this.items.filter((item) => !drop.has(item.id));
  }

  reset(): void {
    this.requestId += 1;
    this.items = [];
    this.next = null;
    this.loaded = false;
    this.loading = false;
    this.error = "";
  }
}

/** Same order the catalog uses: `taken_at DESC, id DESC`. */
function isNewer(a: AssetSummary, b: AssetSummary): boolean {
  return a.takenAt > b.takenAt || (a.takenAt === b.takenAt && a.id > b.id);
}
