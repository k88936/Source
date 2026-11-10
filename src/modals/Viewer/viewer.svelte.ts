import type {Snippet} from 'svelte';
import type {ViewerItem} from '#lib/components/viewer/Viewer/viewer-item';

/**
 * What the viewer screen shows. The page that opens it hands over its own list,
 * so the viewer never fetches: paging and the current index stay in one place.
 *
 * A timeline and a local album differ only in the list they expose and in the
 * extra header controls, so both open the same viewer and inject what they have.
 */
export type ViewerSource = {
    items: () => ViewerItem[];
    total: () => number;
    loadMore: () => void;
    /** The asset to open on. */
    index: number;
    /** Extra header controls; a local album has none. */
    actions?: Snippet<[{index: number}]>;
};

class ViewerState {
    source = $state<ViewerSource | null>(null);
    index = $state(0);

    show(source: ViewerSource): void {
        this.source = source;
        this.index = source.index;
    }
}

export const viewer = new ViewerState();
