import type {Snippet} from 'svelte';
import type {AssetSummary} from '#lib/api/bindings/AssetSummary';
import {viewer} from '../../../modals/Viewer/viewer.svelte';
import {Search} from '../search/search.svelte';
import {Selection} from '../selection/selection.svelte';
import {timelineItem} from '../viewer/Viewer/viewer-item';
import {Timeline, type PageLoader} from './timeline.svelte';

/** The viewer screen every list opens; a tab's own param list narrows to this. */
type ViewerNavigation = {navigate(name: 'Viewer'): void};

/**
 * The list, its selection and its search: one page's worth of state.
 *
 * Photos and an album differ only in what they fetch and in what a move does to
 * their list, so both host this instead of repeating the same three objects and
 * the same open/back wiring.
 */
export class AssetBrowser {
    readonly timeline: Timeline;
    readonly selection = new Selection();
    readonly search: Search;

    constructor(
        loader: PageLoader,
        {
            scroll,
            folder = () => null,
        }: {
            /** The page's scroll container, so search reopens where it was left. */
            scroll: () => HTMLElement | undefined;
            /** Album the results are confined to, or `null` for the whole library. */
            folder?: () => string | null;
        },
    ) {
        this.timeline = new Timeline(loader);
        this.search = new Search(folder, this.selection, scroll);
    }

    /** The list on screen: the timeline, or the results while search is on. */
    get shown(): Timeline {
        return this.search.open ? this.search.timeline : this.timeline;
    }

    open(asset: AssetSummary, navigation: ViewerNavigation, actions: Snippet<[{index: number}]>): void {
        const list = this.shown;
        const index = list.items.findIndex((item) => item.id === asset.id);
        if (index < 0) return;
        viewer.show({
            items: () => list.items.map(timelineItem),
            total: () => list.total,
            loadMore: () => void list.loadMore(),
            index,
            actions,
        });
        navigation.navigate('Viewer');
    }

    /** The back key leaves the mode the page is in, before it leaves the page. */
    back(): void {
        if (this.search.open) this.search.exit();
        else this.selection.exit();
    }

    dispose(): void {
        this.search.dispose();
    }
}
