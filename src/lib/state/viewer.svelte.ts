import type {LocalAsset} from '#lib/api/bindings/LocalAsset';
import type {Timeline} from './timeline.svelte';

/**
 * What the viewer screen shows. The page that opens it fills this in first, so
 * the viewer itself never fetches: it points at the very list it was opened from,
 * which keeps paging, favourites and the current index in one place.
 */
export type ViewerSource =
    | {kind: 'timeline'; timeline: Timeline}
    | {kind: 'local'; items: () => LocalAsset[]; total: () => number; loadMore: () => void};

class ViewerSourceState {
    source = $state<ViewerSource | null>(null);
    index = $state(0);

    showTimeline(timeline: Timeline, index: number): void {
        this.source = {kind: 'timeline', timeline};
        this.index = index;
    }

    showLocalAlbum(
        items: () => LocalAsset[],
        total: () => number,
        loadMore: () => void,
        index: number,
    ): void {
        this.source = {kind: 'local', items, total, loadMore};
        this.index = index;
    }
}

export const viewerSource = new ViewerSourceState();
